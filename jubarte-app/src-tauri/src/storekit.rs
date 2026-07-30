//! StoreKit 2 bridge — Rust side.
//!
//! The actual StoreKit work lives in the Swift static library under
//! `storekit/` (linked by `swift-rs`, see `build.rs`). This module is the typed
//! JSON contract plus the Tauri commands the webview calls.
//!
//! Wire protocol: every Swift entry point returns a JSON string shaped as a
//! `{"ok": Bool, ...}` envelope. [`decode`] maps that onto `Result<T, String>`
//! — `ok:false` becomes `Err(error)`, `ok:true` re-parses the payload into `T`.
//!
//! Threading: StoreKit 2 is async and the Swift side blocks a background thread
//! on a semaphore to expose it as a synchronous C call. Therefore every command
//! runs its Swift call inside `spawn_blocking`, never on Tauri's UI thread.
//!
//! Trust boundary: the booleans returned here (`active`, `expiration_date`) are
//! a convenience for the UI. The authoritative signal is the signed `jws`, which
//! the backend must verify against Apple's public keys before granting access.

use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};

#[cfg(target_os = "macos")]
use swift_rs::{SRString, swift};

// FFI declarations — defined in storekit/Sources/jubarte-storekit/StoreKitBridge.swift.
#[cfg(target_os = "macos")]
swift!(fn jubarte_fetch_products(ids_json: &SRString) -> SRString);
#[cfg(target_os = "macos")]
swift!(fn jubarte_purchase(product_id: &SRString) -> SRString);
#[cfg(target_os = "macos")]
swift!(fn jubarte_current_entitlement(product_id: &SRString) -> SRString);
#[cfg(target_os = "macos")]
swift!(fn jubarte_restore() -> SRString);
#[cfg(target_os = "macos")]
swift!(fn jubarte_start_transaction_listener());

/// Shown when a StoreKit command is invoked on a non-Store (e.g. Windows) build.
#[cfg(not(target_os = "macos"))]
const NOT_MACOS: &str = "jubarte: in-app purchase is only available in the Mac App Store build";

// ---------------------------------------------------------------------------
// Typed mirror of the Swift JSON payloads.
// ---------------------------------------------------------------------------

/// One purchasable product (the annual subscription, in practice).
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Product {
    pub id: String,
    #[serde(rename = "displayName")]
    pub display_name: String,
    pub description: String,
    /// Localized, currency-formatted price, e.g. `"$99.99"`.
    #[serde(rename = "displayPrice")]
    pub display_price: String,
    /// Raw decimal as a string, e.g. `"99.99"`.
    pub price: String,
    #[serde(rename = "isSubscription")]
    pub is_subscription: bool,
    /// `"day" | "week" | "month" | "year"` when this is a subscription.
    #[serde(rename = "subscriptionPeriodUnit")]
    pub subscription_period_unit: Option<String>,
    #[serde(rename = "subscriptionPeriodValue")]
    pub subscription_period_value: Option<i64>,
}

/// Payload of a successful `jubarte_fetch_products` call.
#[derive(Debug, Deserialize)]
struct ProductsPayload {
    products: Vec<Product>,
}

/// Outcome of a purchase attempt.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct PurchaseResult {
    /// `"purchased" | "pending" | "userCancelled" | "verificationFailed"`.
    pub status: String,
    #[serde(rename = "productId")]
    pub product_id: Option<String>,
    #[serde(rename = "transactionId")]
    pub transaction_id: Option<String>,
    #[serde(rename = "originalTransactionId")]
    pub original_transaction_id: Option<String>,
    /// Epoch seconds; `None` for non-expiring products.
    #[serde(rename = "expirationDate")]
    pub expiration_date: Option<f64>,
    /// Signed transaction (JWS) for server-side verification.
    pub jws: Option<String>,
}

/// Current entitlement snapshot for a product.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct EntitlementStatus {
    pub active: bool,
    #[serde(rename = "productId")]
    pub product_id: Option<String>,
    #[serde(rename = "transactionId")]
    pub transaction_id: Option<String>,
    #[serde(rename = "expirationDate")]
    pub expiration_date: Option<f64>,
    pub jws: Option<String>,
}

/// Minimal envelope read first to branch on success/failure.
#[derive(Debug, Deserialize)]
struct Envelope {
    ok: bool,
    #[serde(default)]
    error: Option<String>,
}

/// Map a Swift JSON envelope onto a `Result`.
///
/// - malformed JSON → `Err` (surfaced, never silently swallowed);
/// - `ok:false` → `Err(error)` (or a fallback if `error` is absent);
/// - `ok:true` → re-parse the whole document into `T`.
fn decode<T: DeserializeOwned>(json: &str) -> Result<T, String> {
    let envelope: Envelope = serde_json::from_str(json)
        .map_err(|e| format!("jubarte: malformed StoreKit response: {e}"))?;
    if !envelope.ok {
        return Err(envelope
            .error
            .unwrap_or_else(|| "jubarte: unknown StoreKit error".to_string()));
    }
    serde_json::from_str(json)
        .map_err(|e| format!("jubarte: could not parse StoreKit payload: {e}"))
}

// ---------------------------------------------------------------------------
// Swift invocation (macOS only).
// ---------------------------------------------------------------------------

/// Run a Swift StoreKit call on a blocking worker thread and decode its result.
/// `f` performs the actual (semaphore-bridged, blocking) Swift call.
#[cfg(target_os = "macos")]
async fn run_storekit<T, F>(what: &'static str, f: F) -> Result<T, String>
where
    T: DeserializeOwned + Send + 'static,
    F: FnOnce() -> SRString + Send + 'static,
{
    tauri::async_runtime::spawn_blocking(move || decode::<T>(f().as_str()))
        .await
        .map_err(|_| format!("jubarte: StoreKit {what} call panicked"))?
}

// ---------------------------------------------------------------------------
// Tauri commands.
// ---------------------------------------------------------------------------

/// Fetch metadata for the given product identifiers.
#[tauri::command]
pub async fn storekit_fetch_products(product_ids: Vec<String>) -> Result<Vec<Product>, String> {
    #[cfg(target_os = "macos")]
    {
        let ids_json = serde_json::to_string(&product_ids).map_err(|e| e.to_string())?;
        let payload: ProductsPayload = run_storekit("fetch_products", move || {
            let arg: SRString = ids_json.as_str().into();
            unsafe { jubarte_fetch_products(&arg) }
        })
        .await?;
        Ok(payload.products)
    }
    #[cfg(not(target_os = "macos"))]
    {
        let _ = product_ids;
        Err(NOT_MACOS.into())
    }
}

/// Start a purchase and present the App Store payment sheet.
#[tauri::command]
pub async fn storekit_purchase(product_id: String) -> Result<PurchaseResult, String> {
    #[cfg(target_os = "macos")]
    {
        run_storekit("purchase", move || {
            let arg: SRString = product_id.as_str().into();
            unsafe { jubarte_purchase(&arg) }
        })
        .await
    }
    #[cfg(not(target_os = "macos"))]
    {
        let _ = product_id;
        Err(NOT_MACOS.into())
    }
}

/// Report the current entitlement for a product.
#[tauri::command]
pub async fn storekit_current_entitlement(product_id: String) -> Result<EntitlementStatus, String> {
    #[cfg(target_os = "macos")]
    {
        run_storekit("current_entitlement", move || {
            let arg: SRString = product_id.as_str().into();
            unsafe { jubarte_current_entitlement(&arg) }
        })
        .await
    }
    #[cfg(not(target_os = "macos"))]
    {
        let _ = product_id;
        Err(NOT_MACOS.into())
    }
}

/// Restore purchases (`AppStore.sync`) and report the first active entitlement.
#[tauri::command]
pub async fn storekit_restore() -> Result<EntitlementStatus, String> {
    #[cfg(target_os = "macos")]
    {
        run_storekit("restore", move || unsafe { jubarte_restore() }).await
    }
    #[cfg(not(target_os = "macos"))]
    {
        Err(NOT_MACOS.into())
    }
}

/// The single product Jubarte sells (mirrors `PRODUCT_ID` in src/storekit.js).
pub const PRODUCT_ID: &str = "com.jandira.jubarte.pro.yearly";

/// Short-TTL cache for the quota gate's entitlement check. A permanent
/// session flag would keep access after a refund/revocation until restart
/// (CodeRabbit #3623452491). `false` is never cached, so a purchase made
/// mid-session is still picked up by the next redline attempt.
#[cfg(target_os = "macos")]
const ENTITLEMENT_CACHE_TTL: std::time::Duration = std::time::Duration::from_secs(60);

#[cfg(target_os = "macos")]
struct EntitlementCache {
    active_until: std::sync::Mutex<Option<std::time::Instant>>,
}

#[cfg(target_os = "macos")]
static ENTITLEMENT_CACHE: EntitlementCache = EntitlementCache {
    active_until: std::sync::Mutex::new(None),
};

/// Does the user hold an active subscription right now? Used by the Rust-side
/// free-quota gate. Errors (unsigned dev build, StoreKit unavailable) read as
/// "not entitled" — the free quota still applies there.
pub async fn is_entitled_for_gate() -> bool {
    #[cfg(target_os = "macos")]
    {
        {
            let guard = ENTITLEMENT_CACHE.active_until.lock().unwrap();
            if let Some(until) = *guard {
                if std::time::Instant::now() < until {
                    return true;
                }
            }
        }
        let active = storekit_current_entitlement(PRODUCT_ID.to_string())
            .await
            .map(|s| s.active)
            .unwrap_or(false);
        if active {
            *ENTITLEMENT_CACHE.active_until.lock().unwrap() =
                Some(std::time::Instant::now() + ENTITLEMENT_CACHE_TTL);
        } else {
            *ENTITLEMENT_CACHE.active_until.lock().unwrap() = None;
        }
        active
    }
    #[cfg(not(target_os = "macos"))]
    {
        false
    }
}

/// Install the StoreKit `Transaction.updates` listener. Call once at app
/// startup so renewals and other out-of-band transactions get verified and
/// finished (otherwise StoreKit keeps re-delivering them). No-op off macOS.
pub fn start_transaction_listener() {
    #[cfg(target_os = "macos")]
    unsafe {
        jubarte_start_transaction_listener();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn decodes_products_ok() {
        let json = r#"{"ok":true,"products":[{"id":"com.jandira.jubarte.annual","displayName":"Jubarte Annual","description":"One year of Jubarte","displayPrice":"$99.99","price":"99.99","isSubscription":true,"subscriptionPeriodUnit":"year","subscriptionPeriodValue":1}]}"#;
        let payload: ProductsPayload = decode(json).expect("should decode ok:true products");
        assert_eq!(payload.products.len(), 1);
        let p = &payload.products[0];
        assert_eq!(p.id, "com.jandira.jubarte.annual");
        assert_eq!(p.display_price, "$99.99");
        assert!(p.is_subscription);
        assert_eq!(p.subscription_period_unit.as_deref(), Some("year"));
        assert_eq!(p.subscription_period_value, Some(1));
    }

    #[test]
    fn maps_ok_false_to_err_with_message() {
        let json = r#"{"ok":false,"error":"jubarte: the network is down"}"#;
        let r: Result<ProductsPayload, String> = decode(json);
        assert_eq!(r.err().as_deref(), Some("jubarte: the network is down"));
    }

    #[test]
    fn ok_false_without_error_uses_fallback() {
        let json = r#"{"ok":false}"#;
        let r: Result<EntitlementStatus, String> = decode(json);
        assert!(
            r.unwrap_err().contains("unknown StoreKit error"),
            "should fall back to a generic message"
        );
    }

    #[test]
    fn decodes_purchase_purchased_with_jws() {
        let json = r#"{"ok":true,"status":"purchased","productId":"com.jandira.jubarte.annual","transactionId":"2000000123","originalTransactionId":"2000000123","expirationDate":1789900000.0,"jws":"eyJ.signed.jws"}"#;
        let r: PurchaseResult = decode(json).unwrap();
        assert_eq!(r.status, "purchased");
        assert_eq!(r.transaction_id.as_deref(), Some("2000000123"));
        assert_eq!(r.jws.as_deref(), Some("eyJ.signed.jws"));
        assert_eq!(r.expiration_date, Some(1789900000.0));
    }

    #[test]
    fn decodes_purchase_user_cancelled_without_jws() {
        let json = r#"{"ok":true,"status":"userCancelled"}"#;
        let r: PurchaseResult = decode(json).unwrap();
        assert_eq!(r.status, "userCancelled");
        assert!(r.jws.is_none());
        assert!(r.transaction_id.is_none());
    }

    #[test]
    fn decodes_active_entitlement() {
        let json = r#"{"ok":true,"active":true,"productId":"com.jandira.jubarte.annual","transactionId":"2000000123","expirationDate":1789900000.0,"jws":"eyJ.x"}"#;
        let r: EntitlementStatus = decode(json).unwrap();
        assert!(r.active);
        assert_eq!(r.product_id.as_deref(), Some("com.jandira.jubarte.annual"));
        assert_eq!(r.jws.as_deref(), Some("eyJ.x"));
    }

    #[test]
    fn decodes_inactive_entitlement() {
        let json = r#"{"ok":true,"active":false}"#;
        let r: EntitlementStatus = decode(json).unwrap();
        assert!(!r.active);
        assert!(r.jws.is_none());
    }

    #[test]
    fn malformed_json_is_err() {
        let r: Result<EntitlementStatus, String> = decode("not json at all");
        assert!(
            r.unwrap_err().contains("malformed"),
            "malformed input must surface as an error"
        );
    }

    #[test]
    fn ok_true_with_wrong_shape_is_parse_error() {
        // Envelope ok, but missing required Product fields.
        let json = r#"{"ok":true,"products":[{"id":"x"}]}"#;
        let r: Result<ProductsPayload, String> = decode(json);
        assert!(
            r.unwrap_err().contains("could not parse"),
            "ok:true with incomplete payload must surface a parse error"
        );
    }

    #[test]
    fn empty_products_list_is_ok() {
        let json = r#"{"ok":true,"products":[]}"#;
        let payload: ProductsPayload = decode(json).unwrap();
        assert!(payload.products.is_empty());
    }

    #[test]
    fn decodes_pending_and_verification_failed() {
        let pending: PurchaseResult = decode(r#"{"ok":true,"status":"pending"}"#).unwrap();
        assert_eq!(pending.status, "pending");
        let failed: PurchaseResult =
            decode(r#"{"ok":true,"status":"verificationFailed","jws":null}"#).unwrap();
        assert_eq!(failed.status, "verificationFailed");
    }

    #[test]
    fn product_id_matches_frontend_live_id() {
        // Keep Rust gate + JS PRODUCT_ID in lockstep (deleted ASC ids cannot be reused).
        assert_eq!(PRODUCT_ID, "com.jandira.jubarte.pro.yearly");
    }

    #[test]
    fn locale_independent_price_string_is_dot_decimal() {
        // Mirrors the Swift `product.price.description` contract (gemini #3583605121).
        let json = r#"{"ok":true,"products":[{"id":"p","displayName":"P","description":"d","displayPrice":"R$99,99","price":"99.99","isSubscription":true,"subscriptionPeriodUnit":"year","subscriptionPeriodValue":1}]}"#;
        let payload: ProductsPayload = decode(json).unwrap();
        assert_eq!(payload.products[0].price, "99.99");
        assert!(!payload.products[0].price.contains(','));
    }
}
