#!/usr/bin/env bash
# SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
#
# SPDX-License-Identifier: AGPL-3.0-only
#
# Disk each replaced tool adds to a fresh Ubuntu 24.04 (linux/amd64)
# container, the usual agent sandbox, against jubarte's release binary and
# wheel. Each row is its own container: the bytes `du` counts on / after
# the install minus before, package caches and apt lists removed. Downloads
# about 2 GB, so it runs only with JUBARTE_MEASURE_SIZES=1 (and Docker);
# otherwise it prints the committed table.
set -euo pipefail
cd "$(dirname "$0")"
out=sizes_jubarte_vs_replaced.tsv
if [ "${JUBARTE_MEASURE_SIZES:-0}" != 1 ] || ! command -v docker >/dev/null; then
  echo "skip measuring (set JUBARTE_MEASURE_SIZES=1 with Docker); committed results:"
  cat "$out" download_jubarte.tsv
  exit 0
fi
version=${JUBARTE_VERSION:-0.11.2}
image=ubuntu:24.04
docker pull -q --platform linux/amd64 "$image" >/dev/null

# row LABEL BASE INSTALL: a fresh container; BASE is installed first and
# not counted (Python for the pip rows), then the bytes INSTALL adds.
row() {
  local label=$1 base=$2 install=$3
  local bytes
  bytes=$(docker run --rm --platform linux/amd64 -e DEBIAN_FRONTEND=noninteractive "$image" bash -c "
    set -e
    apt-get update -qq >/dev/null
    apt-get install -y -qq --no-install-recommends ca-certificates curl $base >/dev/null
    apt-get clean; rm -rf /var/lib/apt/lists/* /root/.cache
    before=\$(du -sxb / 2>/dev/null | cut -f1)
    apt-get update -qq >/dev/null
    $install
    apt-get clean; rm -rf /var/lib/apt/lists/* /root/.cache /root/.npm /tmp/*
    after=\$(du -sxb / 2>/dev/null | cut -f1)
    echo \$((after - before))
  " | tail -1)
  printf '%s\t%s\t%s\n' "$label" "$bytes" "$(awk -v b="$bytes" 'BEGIN { printf "%.0f MB", b / 1e6 }')" | tee -a "$out.new"
}

apt='apt-get install -y -qq --no-install-recommends'
pip='pip install -q --break-system-packages'
printf 'what\tbytes\tsize\n' > "$out.new"
row "LibreOffice (libreoffice-writer-nogui, no recommends): soffice for render, accept, .doc" "" "$apt libreoffice-writer-nogui >/dev/null"
row "LibreOffice (libreoffice, apt default)" "" "apt-get install -y -qq libreoffice >/dev/null"
row "Poppler (poppler-utils): pdftoppm" "" "$apt poppler-utils >/dev/null"
row "pandoc" "" "$apt pandoc >/dev/null"
row "Node.js + npm + docx (docx-js)" "" "$apt nodejs npm >/dev/null && mkdir -p /opt/docxjs && cd /opt/docxjs && npm install -s docx >/dev/null"
row "Python 3 + pip + python-docx (with lxml)" "" "$apt python3 python3-pip >/dev/null && $pip python-docx"
row "python-docx (with lxml), Python already present" "python3 python3-pip" "$pip python-docx"
row "jubarte $version release binary (linux-x86_64)" "" "curl -fsSL https://github.com/jandira-tech/jubarte-redlines/releases/download/v$version/jubarte-$version-linux-x86_64.tar.gz | tar -xz -C /usr/local/bin"
row "jubarte $version wheel (pip), Python already present" "python3 python3-pip" "$pip jubarte-redlines==$version"
mv "$out.new" "$out"

# The release tarball's download size: the Content-Length of the asset the
# binary row above installs, after GitHub's redirect.
url="https://github.com/jandira-tech/jubarte-redlines/releases/download/v$version/jubarte-$version-linux-x86_64.tar.gz"
bytes=$(curl -fsSLI "$url" | awk 'tolower($1)=="content-length:"{n=$2} END{print n}' | tr -d '\r')
printf 'what\tbytes\tsize\njubarte %s release tarball (linux-x86_64) download\t%s\t%s MB\n' \
  "$version" "$bytes" "$(( (bytes + 500000) / 1000000 ))" > download_jubarte.tsv
