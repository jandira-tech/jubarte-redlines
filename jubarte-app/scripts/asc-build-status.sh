#!/usr/bin/env bash
#
# Show the processing state of recent Jubarte builds in App Store Connect.
# PROCESSING = Apple is still validating; VALID = ready to attach to a version;
# INVALID/FAILED = there's a problem to fix before you can submit.
#
# Signs the App Store Connect API JWT with openssl (no Python crypto deps).
#
set -euo pipefail

KEY_ID="D6P299RB86"
ISSUER="04eda013-ed0d-4a50-8bf8-d39149ce7aa6"
APP_APPLE_ID="6790926615"
KEY_PATH="$HOME/.appstoreconnect/private_keys/AuthKey_${KEY_ID}.p8"

[ -f "$KEY_PATH" ] || { echo "ERROR: App Store Connect API key not found at $KEY_PATH"; exit 1; }

# Build an ES256 JWT: openssl signs; Python (stdlib only) assembles + DER→raw.
JWT="$(python3 - "$KEY_PATH" "$KEY_ID" "$ISSUER" <<'PY'
import base64, json, subprocess, sys, time
key_path, kid, iss = sys.argv[1:4]
b64 = lambda d: base64.urlsafe_b64encode(d).rstrip(b"=").decode()
now = int(time.time())
header = b64(json.dumps({"alg": "ES256", "kid": kid, "typ": "JWT"}, separators=(",", ":")).encode())
payload = b64(json.dumps({"iss": iss, "iat": now, "exp": now + 1190, "aud": "appstoreconnect-v1"}, separators=(",", ":")).encode())
signing_input = (header + "." + payload).encode()
der = subprocess.run(["openssl", "dgst", "-sha256", "-sign", key_path],
                     input=signing_input, capture_output=True, check=True).stdout
# Parse DER ECDSA signature (SEQUENCE of two INTEGERs) into raw r||s (32 bytes each).
i = 2 if der[1] < 0x80 else 2 + (der[1] & 0x7F)
assert der[i] == 0x02; rlen = der[i + 1]; r = der[i + 2:i + 2 + rlen]; i += 2 + rlen
assert der[i] == 0x02; slen = der[i + 1]; s = der[i + 2:i + 2 + slen]
raw = r.lstrip(b"\x00").rjust(32, b"\x00") + s.lstrip(b"\x00").rjust(32, b"\x00")
print(header + "." + payload + "." + b64(raw))
PY
)"

curl -s "https://api.appstoreconnect.apple.com/v1/builds?filter%5Bapp%5D=${APP_APPLE_ID}&sort=-uploadedDate&limit=5" \
  -H "Authorization: Bearer $JWT" | python3 -c "
import sys, json
d = json.load(sys.stdin)
if d.get('errors'):
    print('API error:', d['errors']); sys.exit(1)
rows = d.get('data', [])
if not rows:
    print('No builds found yet — processing can take a few minutes after upload.'); sys.exit(0)
print('%-10s %-16s %-28s %s' % ('VERSION', 'PROCESSING', 'UPLOADED', 'EXPIRED'))
for x in rows:
    a = x['attributes']
    print('%-10s %-16s %-28s %s' % (a.get('version'), a.get('processingState'), a.get('uploadedDate'), a.get('expired')))
"
