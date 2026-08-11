// jubarte.pro — marketing landing + legal pages (Privacy Policy, Terms).
// Plain Cloudflare Worker, no dependencies. Content is factual to how Jubarte
// actually handles data (documents processed locally; only Apple-signed
// subscription records reach the backend).

const COMPANY = "Jandira Technologies, LLC";
const CONTACT = "support@jubarte.pro";
const UPDATED = "July 15, 2026";

const CSS = `
:root{--ink:#0b1e2d;--sec:#5a7183;--b500:#25628f;--b700:#194361;--border:#dce6ec;--paper:#f4f8fb}
*{margin:0;padding:0;box-sizing:border-box}
body{font-family:-apple-system,BlinkMacSystemFont,"Segoe UI",Roboto,Helvetica,Arial,sans-serif;color:var(--ink);line-height:1.65;background:#fff}
.wrap{max-width:760px;margin:0 auto;padding:56px 24px 80px}
header.site{border-bottom:1px solid var(--border);background:var(--paper)}
header.site .wrap{padding:20px 24px;display:flex;align-items:center;gap:12px}
.brand{font-weight:700;font-size:18px;letter-spacing:-.01em;text-decoration:none;color:var(--ink)}
.brand span{color:var(--b500)}
nav a{color:var(--sec);text-decoration:none;font-size:14px;margin-left:20px}
nav a:hover{color:var(--b500)}
h1{font-size:30px;letter-spacing:-.02em;margin-bottom:6px}
h2{font-size:18px;margin:30px 0 8px;color:var(--b700)}
p,li{color:#243642;margin-bottom:12px}
.muted{color:var(--sec);font-size:14px}
a{color:var(--b500)}
ul{padding-left:22px;margin-bottom:12px}
.hero h1{font-size:40px;margin-bottom:12px}
.hero p{font-size:18px;color:var(--sec);max-width:52ch}
.card{border:1px solid var(--border);background:var(--paper);padding:22px 24px;margin-top:28px}
footer{border-top:1px solid var(--border);color:var(--sec);font-size:13px}
footer .wrap{padding:24px;display:flex;justify-content:space-between;flex-wrap:wrap;gap:12px}
footer a{color:var(--sec);text-decoration:none;margin-left:16px}
`;

function page(title: string, body: string): Response {
  const html = `<!doctype html><html lang="en"><head>
<meta charset="utf-8"/><meta name="viewport" content="width=device-width,initial-scale=1"/>
<title>${title}</title><style>${CSS}</style></head><body>
<header class="site"><div class="wrap">
  <a class="brand" href="/">jubarte<span>.</span></a>
  <nav style="margin-left:auto"><a href="/">Home</a><a href="/privacy">Privacy</a><a href="/terms">Terms</a></nav>
</div></header>
<main class="wrap">${body}</main>
<footer><div class="wrap">
  <span>© 2026 ${COMPANY}</span>
  <span><a href="/privacy">Privacy Policy</a><a href="/terms">Terms of Use</a><a href="mailto:${CONTACT}">Contact</a></span>
</div></footer></body></html>`;
  return new Response(html, { headers: { "content-type": "text/html; charset=utf-8" } });
}

const LANDING = `
<section class="hero">
  <h1>Word redlines, done right.</h1>
  <p>Drop two Microsoft Word documents and get a clean, tracked-changes redline that
  opens perfectly in Word — insertions, deletions, and moves, exactly where they belong.</p>
</section>
<div class="card">
  <h2 style="margin-top:0">Private by design</h2>
  <p class="muted">Your documents are compared locally on your Mac. Their contents are never
  uploaded to us or anyone else. Jubarte is a native macOS app distributed through the Mac App Store.</p>
</div>
<p style="margin-top:28px" class="muted">Jubarte is a product of ${COMPANY}.
Questions? <a href="mailto:${CONTACT}">${CONTACT}</a>.</p>
`;

const PRIVACY = `
<h1>Privacy Policy</h1>
<p class="muted">Last updated: ${UPDATED}</p>
<p>${COMPANY} (“we”, “us”, “our”) operates Jubarte, a macOS application that compares
Microsoft Word documents and produces tracked-changes redlines. This policy explains what
we do and don’t collect.</p>

<h2>Your documents stay on your device</h2>
<p>Jubarte compares your documents locally, on your Mac. The contents of the documents you
open are <strong>never uploaded</strong> to us or to any third party, and we never see them.</p>

<h2>Subscription data</h2>
<p>Jubarte offers an auto-renewable subscription sold through the Apple App Store. Apple
processes all payments — we never receive your payment card details or your Apple Account
credentials. To verify and manage your subscription, our verification service receives from
Apple a cryptographically-signed transaction record containing subscription identifiers only:
transaction ID, original transaction ID, product identifier, purchase and expiration dates,
and the environment (sandbox or production). We store this entitlement state to confirm your
access. We do not currently process App Store Server Notifications for renewals,
cancellations, or refunds. This record
does <strong>not</strong> contain your name, email address, or any document content.</p>

<h2>No tracking or advertising</h2>
<p>Jubarte contains no third-party advertising, no advertising identifiers, and no third-party
analytics or tracking SDKs.</p>

<h2>Service providers</h2>
<ul>
  <li><strong>Apple Inc.</strong> — processes payments and delivers App Store subscription notifications.</li>
  <li><strong>Cloudflare, Inc.</strong> — hosts our subscription-verification service, including its database.</li>
</ul>

<h2>Data retention</h2>
<p>We retain subscription records for as long as needed to provide the service and to meet
legal, tax, and accounting obligations, after which they are deleted.</p>

<h2>Your rights</h2>
<p>You may contact us at <a href="mailto:${CONTACT}">${CONTACT}</a> to request access to, or
deletion of, the subscription record associated with your Apple transaction. To cancel your
subscription, open System Settings → Apple Account → Subscriptions on your Mac.</p>

<h2>Children</h2>
<p>Jubarte is a professional productivity tool and is not directed to children under 13.</p>

<h2>Changes to this policy</h2>
<p>We may update this policy from time to time. The “last updated” date above reflects the
most recent revision.</p>

<h2>Contact</h2>
<p>${COMPANY} · <a href="mailto:${CONTACT}">${CONTACT}</a></p>
`;

const TERMS = `
<h1>Terms of Use</h1>
<p class="muted">Last updated: ${UPDATED}</p>
<p>These terms govern your use of Jubarte, provided by ${COMPANY}.</p>

<h2>License</h2>
<p>Jubarte is licensed, not sold, to you. Your license to use the app is governed by Apple’s
<a href="https://www.apple.com/legal/internet-services/itunes/dev/stdeula/">Licensed Application
End User License Agreement (EULA)</a>, which applies in addition to these terms.</p>

<h2>Subscription</h2>
<p>Jubarte is offered as an auto-renewable annual subscription through the Apple App Store.
Payment is charged to your Apple Account at confirmation of purchase. The subscription renews
automatically unless cancelled at least 24 hours before the end of the current period. You can
manage or cancel your subscription in System Settings → Apple Account → Subscriptions. Prices
are shown in the app before purchase.</p>

<h2>Acceptable use</h2>
<p>You agree to use Jubarte only for lawful purposes and not to attempt to circumvent its
licensing or interfere with its verification service.</p>

<h2>Disclaimer</h2>
<p>Jubarte is provided “as is”, without warranties of any kind to the maximum extent permitted
by law. While Jubarte aims to produce accurate redlines, you are responsible for reviewing the
output before relying on it.</p>

<h2>Limitation of liability</h2>
<p>To the maximum extent permitted by law, ${COMPANY} shall not be liable for any indirect,
incidental, or consequential damages arising from your use of Jubarte.</p>

<h2>Contact</h2>
<p>${COMPANY} · <a href="mailto:${CONTACT}">${CONTACT}</a></p>
`;

export default {
  async fetch(request: Request): Promise<Response> {
    const { pathname } = new URL(request.url);
    switch (pathname) {
      case "/":
        return page("Jubarte — Word redlines, done right", LANDING);
      case "/privacy":
      case "/privacy/":
        return page("Privacy Policy — Jubarte", PRIVACY);
      case "/terms":
      case "/terms/":
        return page("Terms of Use — Jubarte", TERMS);
      case "/robots.txt":
        return new Response("User-agent: *\nAllow: /\n", {
          headers: { "content-type": "text/plain" },
        });
      default:
        return new Response("Not found", { status: 404 });
    }
  },
};
