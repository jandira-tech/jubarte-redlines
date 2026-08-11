#!/usr/bin/env python3
"""Minimal App Store Connect API client using ES256 JWT auth (manual signing,
avoids relying on PyJWT's crypto backend registration).

Credentials are never committed (SECURITY.md): they come from the environment
(ASC_KEY_PATH / ASC_ISSUER_ID / ASC_KEY_ID) or, failing that, from the local
untracked file ~/.appstoreconnect/jubarte.json with keys
{"key_path": ..., "issuer_id": ..., "key_id": ...}.
"""
import base64, json, os, sys, time, urllib.request, urllib.error
from pathlib import Path
from cryptography.hazmat.primitives import hashes, serialization
from cryptography.hazmat.primitives.asymmetric import ec
from cryptography.hazmat.primitives.asymmetric.utils import decode_dss_signature

BASE = "https://api.appstoreconnect.apple.com/v1"
_CRED_FILE = Path.home() / ".appstoreconnect" / "jubarte.json"


def _credentials():
    env = (os.environ.get("ASC_KEY_PATH"), os.environ.get("ASC_ISSUER_ID"),
           os.environ.get("ASC_KEY_ID"))
    if all(env):
        return env
    if _CRED_FILE.exists():
        c = json.loads(_CRED_FILE.read_text())
        return c["key_path"], c["issuer_id"], c["key_id"]
    raise SystemExit(
        "ASC credentials not configured: set ASC_KEY_PATH/ASC_ISSUER_ID/ASC_KEY_ID "
        f"or create {_CRED_FILE}")


KEY_PATH, ISSUER_ID, KEY_ID = _credentials()

def b64url(data: bytes) -> str:
    return base64.urlsafe_b64encode(data).rstrip(b"=").decode()

def make_token():
    with open(KEY_PATH, "rb") as f:
        private_key = serialization.load_pem_private_key(f.read(), password=None)
    now = int(time.time())
    header = {"alg": "ES256", "kid": KEY_ID, "typ": "JWT"}
    payload = {"iss": ISSUER_ID, "iat": now, "exp": now + 1190, "aud": "appstoreconnect-v1"}
    signing_input = b64url(json.dumps(header, separators=(",", ":")).encode()) + "." + \
                    b64url(json.dumps(payload, separators=(",", ":")).encode())
    der_sig = private_key.sign(signing_input.encode(), ec.ECDSA(hashes.SHA256()))
    r, s = decode_dss_signature(der_sig)
    raw_sig = r.to_bytes(32, "big") + s.to_bytes(32, "big")
    return signing_input + "." + b64url(raw_sig)

def call(method, path, body=None, params=None):
    token = make_token()
    url = BASE + path
    if params:
        from urllib.parse import urlencode
        url += "?" + urlencode(params, doseq=True)
    data = json.dumps(body).encode() if body is not None else None
    req = urllib.request.Request(url, data=data, method=method)
    req.add_header("Authorization", f"Bearer {token}")
    req.add_header("Content-Type", "application/json")
    try:
        with urllib.request.urlopen(req, timeout=30) as resp:
            raw = resp.read()
            return resp.status, (json.loads(raw) if raw else {})
    except urllib.error.HTTPError as e:
        raw = e.read()
        try:
            return e.code, json.loads(raw)
        except Exception:
            return e.code, {"raw": raw.decode(errors="replace")}

if __name__ == "__main__":
    method = sys.argv[1]
    path = sys.argv[2]
    body = json.loads(sys.argv[3]) if len(sys.argv) > 3 and sys.argv[3] else None
    status, resp = call(method, path, body)
    print(f"HTTP {status}")
    print(json.dumps(resp, indent=2))
