#!/usr/bin/env python3
"""Resubmit the Mac app for review WITH its In-App Purchase attached.

This exists because the 2026-07-28 rejection (2.1(b)) was caused by a review
submission that contained only the app-version item: the "Jubarte Pro Yearly"
subscription was never added, so App Review could not evaluate it and rejected
the whole submission.

Run with:  uv run --with cryptography python3 scripts/asc-resubmit.py [--submit]

Without --submit it stops right before the irreversible final step and prints
what it would have done.
"""

import sys
from importlib.machinery import SourceFileLoader
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
asc = SourceFileLoader("asc", str(ROOT / ".asc_client.py")).load_module()

APP_ID = "6790926615"
VERSION_ID = "a0194c95-7621-42b7-9e73-ec210f1c3fc5"
SUBSCRIPTION_ID = "6791004310"
PLATFORM = "MAC_OS"
DO_SUBMIT = "--submit" in sys.argv


def step(msg):
    print(f"\n\033[1;34m▸ {msg}\033[0m")


def call(method, path, body=None, params=None, ok=(200, 201, 204)):
    s, r = asc.call(method, path, body, params)
    if s not in ok:
        print(f"  HTTP {s}: {r}")
        raise SystemExit(f"aborted at {method} {path}")
    return r


def main(version_string, build_id):
    step(f"1/7  Point version {VERSION_ID} at {version_string}")
    call("PATCH", f"/appStoreVersions/{VERSION_ID}", {
        "data": {"type": "appStoreVersions", "id": VERSION_ID,
                 "attributes": {"versionString": version_string}}})

    step("2/7  Declare export compliance on the build")
    # Without this the build cannot be attached to a version item.
    call("PATCH", f"/builds/{build_id}", {
        "data": {"type": "builds", "id": build_id,
                 "attributes": {"usesNonExemptEncryption": False}}})

    step("3/7  Attach the build to the version")
    call("PATCH", f"/appStoreVersions/{VERSION_ID}/relationships/build", {
        "data": {"type": "builds", "id": build_id}})

    step("4/7  Cancel any submission still holding the version")
    subs = call("GET", f"/apps/{APP_ID}/reviewSubmissions",
                params={"filter[platform]": PLATFORM})
    for s in subs.get("data", []):
        if s["attributes"]["state"] in ("UNRESOLVED_ISSUES", "READY_FOR_REVIEW",
                                        "WAITING_FOR_REVIEW", "IN_REVIEW"):
            print(f"  cancelling {s['id']} ({s['attributes']['state']})")
            call("PATCH", f"/reviewSubmissions/{s['id']}", {
                "data": {"type": "reviewSubmissions", "id": s["id"],
                         "attributes": {"canceled": True}}})

    step("5/7  Create a fresh review submission")
    sub = call("POST", "/reviewSubmissions", {
        "data": {"type": "reviewSubmissions",
                 "relationships": {"app": {"data": {"type": "apps", "id": APP_ID}}},
                 "attributes": {"platform": PLATFORM}}})
    sub_id = sub["data"]["id"]
    print(f"  submission {sub_id}")

    step("6/7  Add the app version to the submission")
    call("POST", "/reviewSubmissionItems", {
        "data": {"type": "reviewSubmissionItems",
                 "relationships": {
                     "reviewSubmission": {"data": {"type": "reviewSubmissions", "id": sub_id}},
                     "appStoreVersion": {"data": {"type": "appStoreVersions", "id": VERSION_ID}}}}})
    items = call("GET", f"/reviewSubmissions/{sub_id}/items")
    print(f"  submission now holds {items['meta']['paging']['total']} item(s)")

    # THIS is what was missing in the 0.6.0 submission and caused the 2.1(b)
    # rejection. An auto-renewable subscription CANNOT be added as a
    # reviewSubmissionItem — the API rejects the relationship outright:
    #   'subscription' is not a relationship on the resource 'reviewSubmissionItems'
    # It has its own single-purpose endpoint, POST /v1/subscriptionSubmissions,
    # which submits the subscription for review on its own. Skipping it means
    # the app ships referencing a subscription App Review has never seen.
    step("6b/7  Submit the subscription (separate endpoint — do not skip)")
    if not DO_SUBMIT:
        print("  DRY RUN — would POST /subscriptionSubmissions for "
              f"subscription {SUBSCRIPTION_ID}")
    else:
        sub_state = call("GET", f"/subscriptions/{SUBSCRIPTION_ID}")
        print(f"  subscription state before: "
              f"{sub_state['data']['attributes']['state']}")
        call("POST", "/subscriptionSubmissions", {
            "data": {"type": "subscriptionSubmissions",
                     "relationships": {
                         "subscription": {"data": {"type": "subscriptions",
                                                   "id": SUBSCRIPTION_ID}}}}})
        after = call("GET", f"/subscriptions/{SUBSCRIPTION_ID}")
        print(f"  subscription state after:  "
              f"{after['data']['attributes']['state']}")

    step("7/7  Submit for review")
    if not DO_SUBMIT:
        print("  DRY RUN — re-run with --submit to actually submit.")
        print(f"  submission {sub_id} is staged with both items.")
        return
    call("PATCH", f"/reviewSubmissions/{sub_id}", {
        "data": {"type": "reviewSubmissions", "id": sub_id,
                 "attributes": {"submitted": True}}})
    final = call("GET", f"/reviewSubmissions/{sub_id}")
    print(f"  state: {final['data']['attributes']['state']}")


if __name__ == "__main__":
    args = [a for a in sys.argv[1:] if not a.startswith("--")]
    if len(args) != 2:
        raise SystemExit("usage: asc-resubmit.py <version-string> <build-id> [--submit]")
    main(args[0], args[1])
