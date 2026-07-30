// Paywall + free-tier gate. Loads as an ES module (imports storekit.js).
//
// Access model: every install gets FREE_LIMIT (5) redlines for free — counted
// and enforced in Rust (src-tauri/src/quota.rs) — after which the annual
// subscription is required. Entitlement is server-authoritative when online
// (isEntitledVerified), with a graceful fall back to the local Apple-signed
// StoreKit receipt (isEntitled) so a paying customer isn't locked out when the
// backend/network is unreachable. For a locally-run product no client gate is
// tamper-proof; the backend is the record of truth and the deterrent, not DRM.

import {
  fetchProducts,
  isEntitled,
  isEntitledVerified,
  purchase,
  restore,
  verifyWithBackend,
} from "./storekit.js";

const invoke =
  window.__TAURI__?.core?.invoke ||
  ((cmd) => {
    throw new Error(`Tauri core invoke is not available (command: ${cmd})`);
  });

// Legal pages hosted on jubarte.pro (jubarte-site/ Cloudflare Worker).
const TERMS_URL = "https://jubarte.pro/terms";
const PRIVACY_URL = "https://jubarte.pro/privacy";

const $ = (id) => document.getElementById(id);

// Set synchronously so a keyboard-triggered run() during the async startup
// checks is gated (quota is null until loaded → treated as "no access yet").
window.jubarte = {
  entitled: false,
  quota: null, // {used, limit, remaining} once loaded
  requireAccess() {
    if (this.entitled) return true;
    if (this.quota && this.quota.remaining > 0) return true;
    openPaywall();
    return false;
  },
  // Rust rejected a redline with FREE_LIMIT_REACHED (quota spent mid-session).
  gate() {
    refreshQuota().then(() => openPaywall());
  },
  // A redline was produced — re-read the count and refresh the badge.
  noteUse() {
    refreshQuota();
  },
};

const paywall = $("paywall");
let previouslyFocused = null;

function hide() {
  paywall?.setAttribute("hidden", "");
  document.removeEventListener("keydown", onPaywallKeydown, true);
  if (previouslyFocused && typeof previouslyFocused.focus === "function") {
    previouslyFocused.focus();
  }
  previouslyFocused = null;
}

function focusableInPaywall() {
  if (!paywall) return [];
  return [
    ...paywall.querySelectorAll(
      'button:not([hidden]):not([disabled]), [href], input:not([disabled]), select:not([disabled]), textarea:not([disabled]), [tabindex]:not([tabindex="-1"])',
    ),
  ].filter((el) => el.offsetParent !== null || el === document.activeElement);
}

function onPaywallKeydown(e) {
  if (paywall?.hasAttribute("hidden")) return;
  if (e.key === "Escape") {
    const later = $("pw-later");
    if (later && !later.hidden) {
      e.preventDefault();
      hide();
    }
    return;
  }
  if (e.key !== "Tab") return;
  const nodes = focusableInPaywall();
  if (nodes.length === 0) return;
  const first = nodes[0];
  const last = nodes[nodes.length - 1];
  if (e.shiftKey && document.activeElement === first) {
    e.preventDefault();
    last.focus();
  } else if (!e.shiftKey && document.activeElement === last) {
    e.preventDefault();
    first.focus();
  }
}

function unlock() {
  window.jubarte.entitled = true;
  updateBadge();
  hide();
}

async function refreshQuota() {
  try {
    window.jubarte.quota = await invoke("quota_status");
  } catch {
    /* quota command unavailable — keep the previous value */
  }
  updateBadge();
}

function updateBadge() {
  const el = $("free-left");
  if (!el) return;
  const q = window.jubarte.quota;
  if (window.jubarte.entitled || !q) {
    el.hidden = true;
    return;
  }
  el.hidden = false;
  el.textContent =
    q.remaining > 0
      ? `${q.remaining} of ${q.limit} free redlines left`
      : "Free redlines used up — subscribe";
}

/** Show the paywall, with copy + dismissability matched to the quota state. */
function openPaywall() {
  const q = window.jubarte.quota;
  const exhausted = q && q.remaining <= 0;
  const sub = $("pw-sub");
  if (sub) {
    sub.textContent = exhausted
      ? `You've used all ${q.limit} free redlines on this Mac. Subscribe for unlimited tracked-changes redlines that open cleanly in Microsoft Word.`
      : "Unlimited tracked-changes redlines that open cleanly in Microsoft Word.";
  }
  // While free redlines remain the paywall is an offer, not a wall.
  const later = $("pw-later");
  if (later) later.hidden = !(q && q.remaining > 0);
  previouslyFocused = document.activeElement;
  paywall?.removeAttribute("hidden");
  document.addEventListener("keydown", onPaywallKeydown, true);
  // Focus first actionable control (a11y focus trap — CodeRabbit #3584706846).
  const nodes = focusableInPaywall();
  (nodes[0] || paywall)?.focus?.();
  renderProduct();
}

function setStatus(msg, busy = false) {
  const el = $("paywall-status");
  if (!el) return;
  el.textContent = msg;
  el.hidden = !msg;
  el.classList.toggle("busy", busy);
}

let productRendered = false;
async function renderProduct() {
  if (productRendered) return;
  try {
    const [p] = await fetchProducts();
    if (!p) return;
    productRendered = true;
    $("pw-price").textContent = p.displayPrice;
    const unit = p.subscriptionPeriodUnit || "year";
    $("pw-period").textContent = `per ${unit}`;
    $("pw-cta-price").textContent = `${p.displayPrice}/${unit === "year" ? "yr" : unit}`;
    // Real, localized disclosure straight from StoreKit.
    $("pw-terms").textContent =
      `${p.displayName} is an auto-renewable subscription billed ${p.displayPrice} ` +
      `per ${unit}. It renews automatically unless cancelled at least 24 hours ` +
      `before the end of the current period. Manage or cancel anytime in System ` +
      `Settings → Apple ID → Subscriptions.`;
  } catch {
    /* keep the static fallback copy already in the HTML */
  }
}

async function checkAccess() {
  // Dev escape hatch for unsigned builds where StoreKit is unavailable:
  //   localStorage.setItem("jubarte_dev_bypass", "1")
  if (localStorage.getItem("jubarte_dev_bypass") === "1") {
    unlock();
    return;
  }
  // Quota first (local file, fast) so the app is usable immediately; the
  // StoreKit/backend entitlement check may take seconds on a cold start.
  await refreshQuota();
  const q = window.jubarte.quota;
  if (q && q.remaining <= 0) openPaywall();

  let entitled = false;
  try {
    entitled = await isEntitledVerified();
  } catch {
    /* backend/network unreachable — fall through to the local check */
  }
  if (!entitled) {
    try {
      entitled = await isEntitled();
    } catch {
      /* StoreKit unavailable (e.g. unsigned dev build) — stays false */
    }
  }
  if (entitled) unlock();
}

$("pw-subscribe")?.addEventListener("click", async () => {
  setStatus("Contacting the App Store…", true);
  let r;
  try {
    r = await purchase();
  } catch (e) {
    setStatus(`The purchase could not start: ${e}`);
    return;
  }
  if (r.status === "purchased") {
    // Apple's signed receipt is on-device — unlock now. The backend records
    // the subscription best-effort; a transient network failure there must
    // not lock out a customer who just paid.
    setStatus("You're all set — thank you!");
    unlock();
    if (r.jws) verifyWithBackend(r.jws).catch(() => {});
  } else if (r.status === "userCancelled") {
    setStatus("");
  } else if (r.status === "pending") {
    setStatus("Purchase is pending approval. You'll be unlocked once it clears.");
  } else {
    setStatus("The purchase didn't complete. Please try again.");
  }
});

$("pw-restore")?.addEventListener("click", async () => {
  setStatus("Restoring your subscription…", true);
  try {
    const s = await restore();
    if (!s?.active) {
      setStatus("No active subscription found for this Apple ID.");
      return;
    }
    // Prefer backend verification of the signed JWS; fall back to the local
    // StoreKit receipt when the Worker is unreachable (CodeRabbit #3584706864).
    let entitled = true;
    if (s.jws) {
      try {
        const backend = await verifyWithBackend(s.jws);
        entitled = Boolean(backend.entitled);
      } catch {
        entitled = Boolean(s.active);
      }
    }
    if (entitled) {
      setStatus("Restored — welcome back!");
      unlock();
    } else {
      setStatus("No active subscription found for this Apple ID.");
    }
  } catch (e) {
    setStatus(`Restore failed: ${e}`);
  }
});

$("pw-later")?.addEventListener("click", () => {
  setStatus("");
  hide();
});

$("free-left")?.addEventListener("click", openPaywall);

// Open legal links in the default browser (not the app webview) via the existing
// open_path command (`open <url>` on macOS launches the default browser).
const openExternal = (url) => (e) => {
  e.preventDefault();
  invoke("open_path", { path: url }).catch(() => {});
};
$("pw-terms-link")?.addEventListener("click", openExternal(TERMS_URL));
$("pw-privacy-link")?.addEventListener("click", openExternal(PRIVACY_URL));

checkAccess();
