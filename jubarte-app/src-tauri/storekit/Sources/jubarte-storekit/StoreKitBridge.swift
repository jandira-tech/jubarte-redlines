//  StoreKitBridge.swift
//  Jubarte StoreKit 2 bridge
//
//  Four @_cdecl entry points, each returning a JSON string (SRString). Every
//  response is a `{"ok": Bool, ...}` envelope so the Rust side (src/storekit.rs)
//  can map it cleanly onto `Result<T, String>`.
//
//  Concurrency model: StoreKit 2 is async/await. Each entry point turns that
//  into a synchronous C call by kicking off a `Task` and blocking on a
//  `DispatchSemaphore`. This is only safe because the Rust side ALWAYS invokes
//  these from a `spawn_blocking` worker thread — never Tauri's main/UI thread —
//  so nothing that needs the main run loop is starved. `Product.purchase()`
//  presents the App Store sheet; StoreKit dispatches that presentation to the
//  main thread internally, so calling it from the Task here is correct.
//
//  Trust boundary: the fields returned here (`active`, `expirationDate`, …) are
//  a CONVENIENCE for the UI. The authoritative check is the signed `jws`
//  (JWS-signed transaction) which the Rust/backend side must verify against
//  Apple's public keys. Never gate paid features on the client booleans alone.

import Foundation
import StoreKit
import SwiftRs

// MARK: - Response DTOs (Swift-internal; serialized to JSON at the boundary)

private struct ProductDTO: Encodable {
    var id: String
    var displayName: String
    var description: String
    var displayPrice: String
    var price: String
    var isSubscription: Bool
    var subscriptionPeriodUnit: String?
    var subscriptionPeriodValue: Int?
}

private struct ProductsResponse: Encodable {
    var ok: Bool
    var products: [ProductDTO]? = nil
    var error: String? = nil
}

private struct PurchaseResponse: Encodable {
    var ok: Bool
    /// "purchased" | "pending" | "userCancelled" | "verificationFailed"
    var status: String? = nil
    var productId: String? = nil
    var transactionId: String? = nil
    var originalTransactionId: String? = nil
    /// Epoch seconds (Double) — null for non-expiring products.
    var expirationDate: Double? = nil
    /// The JWS-signed transaction, for server-side verification.
    var jws: String? = nil
    var error: String? = nil
}

private struct EntitlementResponse: Encodable {
    var ok: Bool
    var active: Bool? = nil
    var productId: String? = nil
    var transactionId: String? = nil
    var expirationDate: Double? = nil
    var jws: String? = nil
    var error: String? = nil
}

// MARK: - Helpers

/// Encode any DTO to a JSON string. Falls back to a valid error envelope so the
/// Rust side never receives a non-JSON payload.
private func encodeJSON<T: Encodable>(_ value: T) -> String {
    let encoder = JSONEncoder()
    guard
        let data = try? encoder.encode(value),
        let string = String(data: data, encoding: .utf8)
    else {
        return #"{"ok":false,"error":"jubarte: failed to encode StoreKit response"}"#
    }
    return string
}

/// Reference box to hand the async Task's result back across the semaphore.
/// `@unchecked Sendable`: `value` is written by the Task and read after
/// `semaphore.wait()`; the signal/wait pair establishes the happens-before that
/// makes the single hand-off safe (the read never races the write).
private final class StringBox: @unchecked Sendable {
    var value = #"{"ok":false,"error":"jubarte: no StoreKit result produced"}"#
}

/// Run an async operation to completion on a background thread and return its
/// JSON string synchronously. The caller MUST already be off the main thread.
///
/// `timeoutSeconds` bounds the wait for non-interactive calls so a wedged
/// StoreKit operation can't park a `spawn_blocking` worker forever. Pass `nil`
/// for calls that legitimately present UI and can take minutes (purchase,
/// restore/`AppStore.sync`) — timing those out could abort a real transaction.
/// On timeout we return a fixed string WITHOUT reading `box.value`, so the
/// still-running Task's later write can't race this read.
private func runBlocking(
    timeoutSeconds: Double?,
    _ operation: @escaping () async -> String
) -> SRString {
    let semaphore = DispatchSemaphore(value: 0)
    let box = StringBox()
    Task {
        box.value = await operation()
        semaphore.signal()
    }
    if let timeoutSeconds {
        if semaphore.wait(timeout: .now() + timeoutSeconds) == .timedOut {
            return SRString(#"{"ok":false,"error":"jubarte: StoreKit timed out"}"#)
        }
    } else {
        semaphore.wait()
    }
    return SRString(box.value)
}

/// Serializes purchase attempts so a double-clicked "Subscribe" can't present
/// two payment sheets or spawn duplicate in-flight purchases.
private final class PurchaseGate: @unchecked Sendable {
    static let shared = PurchaseGate()
    private let lock = NSLock()
    private var inFlight = false

    /// Returns true if the caller acquired the gate (must `leave()` after).
    func tryEnter() -> Bool {
        lock.lock()
        defer { lock.unlock() }
        if inFlight { return false }
        inFlight = true
        return true
    }

    func leave() {
        lock.lock()
        inFlight = false
        lock.unlock()
    }
}

/// Owns the single, app-lifetime `Transaction.updates` listener.
private final class UpdatesListener: @unchecked Sendable {
    static let shared = UpdatesListener()
    private let lock = NSLock()
    private var task: Task<Void, Never>?

    @available(macOS 12.0, *)
    func start() {
        lock.lock()
        defer { lock.unlock() }
        if task != nil { return }
        task = Task.detached {
            for await result in Transaction.updates {
                guard case .verified(let transaction) = result else { continue }
                // Finish so StoreKit stops re-delivering (renewals, Ask-to-Buy
                // approvals, other-device purchases, refunds/revocations). Live
                // entitlement is read on demand via jubarte_current_entitlement;
                // pushing the change into the webview needs a Swift→Rust
                // callback channel and is a documented follow-up.
                await transaction.finish()
            }
        }
    }
}

/// `Product.products(for:)` can wedge indefinitely when the app has no usable
/// App Store context (unsigned dev build, blocked network). Inside a purchase —
/// which is deliberately un-timed because the payment sheet may stay open for
/// minutes — that lookup must still be bounded, or the Subscribe button hangs
/// forever with no sheet ever appearing. Races the lookup against a deadline.
/// Real deadline for product lookup: resume as soon as the timer fires without
/// waiting for a wedged `Product.products(for:)` child to join
/// (CodeRabbit #3623452495). Late lookup results are discarded.
@available(macOS 12.0, *)
private func productsWithTimeout(
    _ ids: [String], seconds: Double
) async throws -> [Product] {
    final class Once: @unchecked Sendable {
        private let lock = NSLock()
        private var taken = false
        func take() -> Bool {
            lock.lock()
            defer { lock.unlock() }
            if taken { return false }
            taken = true
            return true
        }
    }
    let once = Once()
    return try await withCheckedThrowingContinuation { cont in
        let work = Task {
            do {
                let products = try await Product.products(for: ids)
                if once.take() {
                    cont.resume(returning: products)
                }
            } catch {
                if once.take() {
                    cont.resume(throwing: error)
                }
            }
        }
        Task {
            try? await Task.sleep(nanoseconds: UInt64(seconds * 1_000_000_000))
            if once.take() {
                work.cancel()
                cont.resume(
                    throwing: NSError(
                        domain: "jubarte", code: 408,
                        userInfo: [
                            NSLocalizedDescriptionKey:
                                "timed out looking up the product — In-App Purchase may be unavailable in this build"
                        ]))
            }
        }
    }
}

@available(macOS 12.0, *)
private func unitString(_ unit: Product.SubscriptionPeriod.Unit) -> String {
    switch unit {
    case .day: return "day"
    case .week: return "week"
    case .month: return "month"
    case .year: return "year"
    @unknown default: return "unknown"
    }
}

private let unsupportedOSJSON =
    #"{"ok":false,"error":"jubarte: StoreKit 2 requires macOS 12 or newer"}"#

// MARK: - Entry points

/// Fetch product metadata. Argument is a JSON array of product identifiers,
/// e.g. `["com.jandira.jubarte.annual"]`.
@_cdecl("jubarte_fetch_products")
public func jubarteFetchProducts(_ idsJson: SRString) -> SRString {
    guard #available(macOS 12.0, *) else { return SRString(unsupportedOSJSON) }

    let raw = idsJson.toString()
    let ids = (try? JSONDecoder().decode([String].self, from: Data(raw.utf8))) ?? []

    return runBlocking(timeoutSeconds: 30) {
        do {
            // Bound the lookup (unsigned / no App Store context can wedge).
            let products = try await productsWithTimeout(ids, seconds: 25)
            let dtos = products.map { product -> ProductDTO in
                let sub = product.subscription
                return ProductDTO(
                    id: product.id,
                    displayName: product.displayName,
                    description: product.description,
                    displayPrice: product.displayPrice,
                    // Locale-independent decimal (always `.`); `"\(price)"` can
                    // emit commas in European locales and break JS/Rust parses.
                    price: product.price.description,
                    isSubscription: sub != nil,
                    subscriptionPeriodUnit: sub.map { unitString($0.subscriptionPeriod.unit) },
                    subscriptionPeriodValue: sub.map { $0.subscriptionPeriod.value }
                )
            }
            return encodeJSON(ProductsResponse(ok: true, products: dtos))
        } catch {
            return encodeJSON(ProductsResponse(
                ok: false, error: "jubarte: \(error.localizedDescription)"))
        }
    }
}

/// Start a purchase for the given product identifier and present the App Store
/// payment sheet.
@_cdecl("jubarte_purchase")
public func jubartePurchase(_ productIdStr: SRString) -> SRString {
    guard #available(macOS 12.0, *) else { return SRString(unsupportedOSJSON) }

    let productId = productIdStr.toString()

    // Serialize concurrent purchase attempts (e.g. a double-clicked button) so
    // StoreKit isn't asked to present two payment sheets at once.
    guard PurchaseGate.shared.tryEnter() else {
        return SRString(#"{"ok":false,"error":"jubarte: a purchase is already in progress"}"#)
    }
    // No timeout: a payment sheet legitimately takes as long as the user needs.
    let result = runBlocking(timeoutSeconds: nil) {
        do {
            let products = try await productsWithTimeout([productId], seconds: 30)
            guard let product = products.first else {
                return encodeJSON(PurchaseResponse(
                    ok: false,
                    error: "jubarte: product not found: \(productId) — "
                        + "In-App Purchase may be unavailable in this build"))
            }

            let result = try await product.purchase()
            switch result {
            case .success(let verification):
                switch verification {
                case .verified(let transaction):
                    let response = PurchaseResponse(
                        ok: true,
                        status: "purchased",
                        productId: transaction.productID,
                        transactionId: "\(transaction.id)",
                        originalTransactionId: "\(transaction.originalID)",
                        expirationDate: transaction.expirationDate?.timeIntervalSince1970,
                        jws: verification.jwsRepresentation
                    )
                    // Acknowledge so StoreKit stops re-delivering this
                    // transaction. The backend still verifies the JWS.
                    await transaction.finish()
                    return encodeJSON(response)
                case .unverified(_, let error):
                    return encodeJSON(PurchaseResponse(
                        ok: false,
                        status: "verificationFailed",
                        error: "jubarte: transaction failed verification: \(error)"))
                }
            case .pending:
                // Deferred (e.g. Ask to Buy). No transaction yet.
                return encodeJSON(PurchaseResponse(ok: true, status: "pending"))
            case .userCancelled:
                return encodeJSON(PurchaseResponse(ok: true, status: "userCancelled"))
            @unknown default:
                return encodeJSON(PurchaseResponse(
                    ok: false, error: "jubarte: unknown purchase result"))
            }
        } catch {
            return encodeJSON(PurchaseResponse(
                ok: false, error: "jubarte: \(error.localizedDescription)"))
        }
    }
    PurchaseGate.shared.leave()
    return result
}

/// Report whether the user currently holds an active entitlement for the given
/// product identifier, based on `Transaction.currentEntitlements`.
@_cdecl("jubarte_current_entitlement")
public func jubarteCurrentEntitlement(_ productIdStr: SRString) -> SRString {
    guard #available(macOS 12.0, *) else { return SRString(unsupportedOSJSON) }

    let productId = productIdStr.toString()

    return runBlocking(timeoutSeconds: 30) {
        for await result in Transaction.currentEntitlements {
            guard case .verified(let transaction) = result else { continue }
            guard transaction.productID == productId, transaction.revocationDate == nil else {
                continue
            }
            // Presence in currentEntitlements means the customer is entitled
            // right now — INCLUDING during a billing grace period, when
            // `expirationDate` is already in the past but access continues.
            // Gating on `expirationDate > now` here would wrongly lock out a
            // paying customer whose renewal is in billing retry. Report the
            // date as information only.
            return encodeJSON(EntitlementResponse(
                ok: true,
                active: true,
                productId: transaction.productID,
                transactionId: "\(transaction.id)",
                expirationDate: transaction.expirationDate?.timeIntervalSince1970,
                jws: result.jwsRepresentation
            ))
        }
        return encodeJSON(EntitlementResponse(ok: true, active: false))
    }
}

/// Force a sync with the App Store (Restore Purchases) and report the first
/// active entitlement found afterwards.
@_cdecl("jubarte_restore")
public func jubarteRestore() -> SRString {
    guard #available(macOS 12.0, *) else { return SRString(unsupportedOSJSON) }

    // No timeout: AppStore.sync() can present an Apple ID sign-in sheet.
    return runBlocking(timeoutSeconds: nil) {
        do {
            try await AppStore.sync()
        } catch {
            return encodeJSON(EntitlementResponse(
                ok: false, error: "jubarte: restore failed: \(error.localizedDescription)"))
        }

        for await result in Transaction.currentEntitlements {
            guard case .verified(let transaction) = result else { continue }
            guard transaction.revocationDate == nil else { continue }
            // Any entitlement still present in currentEntitlements is valid now
            // (grace period included) — report it without an expirationDate gate.
            return encodeJSON(EntitlementResponse(
                ok: true,
                active: true,
                productId: transaction.productID,
                transactionId: "\(transaction.id)",
                expirationDate: transaction.expirationDate?.timeIntervalSince1970,
                jws: result.jwsRepresentation
            ))
        }
        return encodeJSON(EntitlementResponse(ok: true, active: false))
    }
}

/// Install the app-lifetime `Transaction.updates` listener. Call once at launch
/// (from Rust) so renewals and other out-of-band transactions are verified and
/// finished. Idempotent.
@_cdecl("jubarte_start_transaction_listener")
public func jubarteStartTransactionListener() {
    guard #available(macOS 12.0, *) else { return }
    UpdatesListener.shared.start()
}
