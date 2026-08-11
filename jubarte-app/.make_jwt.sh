#!/bin/bash
# Generates an ES256 JWT for App Store Connect API auth using openssl + base64 only.
set -euo pipefail

# Credentials are never committed (SECURITY.md): env first, then the local
# untracked ~/.appstoreconnect/jubarte.json ({"key_path","issuer_id","key_id"}).
CRED_FILE="$HOME/.appstoreconnect/jubarte.json"
KEY_PATH="${ASC_KEY_PATH:-}"
ISSUER_ID="${ASC_ISSUER_ID:-}"
KEY_ID="${ASC_KEY_ID:-}"
if { [ -z "$KEY_PATH" ] || [ -z "$ISSUER_ID" ] || [ -z "$KEY_ID" ]; } && [ -f "$CRED_FILE" ]; then
  KEY_PATH="${KEY_PATH:-$(python3 -c 'import json,os;print(json.load(open(os.path.expanduser("~/.appstoreconnect/jubarte.json")))["key_path"])')}"
  ISSUER_ID="${ISSUER_ID:-$(python3 -c 'import json,os;print(json.load(open(os.path.expanduser("~/.appstoreconnect/jubarte.json")))["issuer_id"])')}"
  KEY_ID="${KEY_ID:-$(python3 -c 'import json,os;print(json.load(open(os.path.expanduser("~/.appstoreconnect/jubarte.json")))["key_id"])')}"
fi
if [ -z "$KEY_PATH" ] || [ -z "$ISSUER_ID" ] || [ -z "$KEY_ID" ]; then
  echo "ERROR: set ASC_KEY_PATH/ASC_ISSUER_ID/ASC_KEY_ID or create $CRED_FILE" >&2
  exit 1
fi

b64url() {
  openssl base64 -A | tr '+/' '-_' | tr -d '='
}

now=$(date +%s)
exp=$((now + 1190))

header=$(printf '{"alg":"ES256","kid":"%s","typ":"JWT"}' "$KEY_ID" | b64url)
payload=$(printf '{"iss":"%s","iat":%d,"exp":%d,"aud":"appstoreconnect-v1"}' "$ISSUER_ID" "$now" "$exp" | b64url)
signing_input="${header}.${payload}"

# Sign with ES256, convert DER signature to raw r||s (64 bytes) format required by JWS.
der_sig=$(printf '%s' "$signing_input" | openssl dgst -sha256 -sign "$KEY_PATH" | openssl base64 -A)
echo "$der_sig" > /tmp/der_sig.b64

python3 - "$signing_input" <<'PYEOF' 2>/dev/null || true
PYEOF

# Use openssl asn1parse to extract r and s from the DER signature, pad to 32 bytes each.
der_sig_bin=$(printf '%s' "$signing_input" | openssl dgst -sha256 -sign "$KEY_PATH" -out /tmp/sig.der; xxd -p -c 256 /tmp/sig.der | tr -d '\n')

# Parse DER: 30 <len> 02 <rlen> <r> 02 <slen> <s>
hex="$der_sig_bin"
# skip "30" and length byte(s)
pos=2
seqlen_byte=${hex:$pos:2}
pos=$((pos+2))
seqlen_val=$((16#$seqlen_byte))
if [ "$seqlen_val" -ge 128 ]; then
  nbytes=$((seqlen_val - 128))
  pos=$((pos + nbytes*2))
fi
# now at 02 <rlen>
tag_r=${hex:$pos:2}; pos=$((pos+2))
rlen_byte=${hex:$pos:2}; pos=$((pos+2))
rlen=$((16#$rlen_byte))
r_hex=${hex:$pos:$((rlen*2))}; pos=$((pos+rlen*2))
tag_s=${hex:$pos:2}; pos=$((pos+2))
slen_byte=${hex:$pos:2}; pos=$((pos+2))
slen=$((16#$slen_byte))
s_hex=${hex:$pos:$((slen*2))}

# strip leading 00 padding if r/s are 33 bytes (sign bit), pad to 32 bytes (64 hex chars)
strip_and_pad() {
  local h="$1"
  while [ ${#h} -gt 64 ] && [ "${h:0:2}" = "00" ]; do
    h="${h:2}"
  done
  while [ ${#h} -lt 64 ]; do
    h="0${h}"
  done
  echo "$h"
}
r_fixed=$(strip_and_pad "$r_hex")
s_fixed=$(strip_and_pad "$s_hex")
raw_sig_hex="${r_fixed}${s_fixed}"

raw_sig_b64=$(echo "$raw_sig_hex" | xxd -r -p | b64url)

echo "${signing_input}.${raw_sig_b64}"
