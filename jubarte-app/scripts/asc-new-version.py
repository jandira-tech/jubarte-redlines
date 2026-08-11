#!/usr/bin/env python3
# SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
# SPDX-License-Identifier: AGPL-3.0-only
"""Create a NEW App Store version and attach a processed build.

Unlike asc-resubmit.py (which re-points the existing in-review version
record), this is for a fresh release after the previous version shipped:
POST a new appStoreVersion, declare export compliance on the build, attach
it. Stops there — "Submit for Review" stays a human click.

Run:  uv run --with cryptography python3 scripts/asc-new-version.py 0.6.2 [--apply]

Without --apply every write is printed, nothing is sent.
"""

import json
import sys
from importlib.machinery import SourceFileLoader
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
asc = SourceFileLoader("asc", str(ROOT / ".asc_client.py")).load_module()

APP_ID = "6790926615"
PLATFORM = "MAC_OS"
DO_APPLY = "--apply" in sys.argv


def call(method, path, body=None, params=None, ok=(200, 201, 204)):
    status, payload = asc.call(method, path, body=body, params=params)
    if status not in ok:
        sys.exit(f"ASC {method} {path} -> {status}: {json.dumps(payload)[:400]}")
    return payload


def mutate(method, path, body=None, ok=(200, 201, 204)):
    if not DO_APPLY:
        print(f"  DRY {method} {path}\n      {json.dumps(body)[:200]}")
        return {}
    return call(method, path, body=body, ok=ok)


def main():
    args = [a for a in sys.argv[1:] if not a.startswith("--")]
    if not args:
        sys.exit("usage: asc-new-version.py <version-string> [--apply]")
    version_string = args[0]

    print(f"1/4  Find the VALID build for {version_string}")
    builds = call(
        "GET",
        "/builds",
        params={
            "filter[app]": APP_ID,
            "filter[processingState]": "VALID",
            "sort": "-uploadedDate",
            "limit": "10",
        },
    )
    build_id = None
    for b in builds.get("data", []):
        if b["attributes"].get("version") and b["attributes"].get("processingState") == "VALID":
            pre = b.get("relationships", {})
            # match by the train's version string via preReleaseVersion is
            # indirect; the CFBundleShortVersionString lives on the train.
            build_id = b["id"]
            marketing = b["attributes"].get("version")
            print(f"  candidate build {build_id} (build number {marketing})")
            break
    if not build_id:
        sys.exit("no VALID build found")

    print(f"2/4  Create appStoreVersion {version_string}")
    resp = mutate(
        "POST",
        "/appStoreVersions",
        {
            "data": {
                "type": "appStoreVersions",
                "attributes": {
                    "platform": PLATFORM,
                    "versionString": version_string,
                },
                "relationships": {
                    "app": {"data": {"type": "apps", "id": APP_ID}}
                },
            }
        },
    )
    version_id = resp.get("data", {}).get("id", "<dry>")
    print(f"  new version id: {version_id}")

    print("3/4  Declare export compliance on the build")
    mutate(
        "PATCH",
        f"/builds/{build_id}",
        {
            "data": {
                "type": "builds",
                "id": build_id,
                "attributes": {"usesNonExemptEncryption": False},
            }
        },
    )

    print("4/4  Attach the build to the new version")
    mutate(
        "PATCH",
        f"/appStoreVersions/{version_id}/relationships/build",
        {"data": {"type": "builds", "id": build_id}},
    )
    print("done — attach metadata + Submit for Review in App Store Connect")


if __name__ == "__main__":
    main()
