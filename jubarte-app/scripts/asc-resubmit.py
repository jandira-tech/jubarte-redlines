#!/usr/bin/env python3
"""Resubmit the Mac app for review WITH its In-App Purchase attached.

This exists because the 2026-07-28 rejection (2.1(b)) was caused by a review
submission that contained only the app-version item: the "Jubarte Pro Yearly"
subscription was never added, so App Review could not evaluate it and rejected
the whole submission.

Run with:  uv run --with cryptography python3 scripts/asc-resubmit.py [--submit]

Without --submit nothing is written to App Store Connect: every PATCH/POST is
printed as a plan instead of executed (reads still run, so the plan reflects
live state).
"""

import sys
from pathlib import Path

from asc_loader import load_asc

ROOT = Path(__file__).resolve().parent.parent
asc = load_asc(ROOT)

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


def mutate(method, path, body=None, ok=(200, 201, 204)):
    """Write gated on --submit: a dry run prints the plan and touches nothing."""
    if not DO_SUBMIT:
        print(f"  DRY RUN — would {method} {path}")
        return None
    return call(method, path, body, ok=ok)


def submission_holds_version(sub_id):
    """True when one of the submission's items is exactly our app version."""
    items = call("GET", f"/reviewSubmissions/{sub_id}/items")
    for item in items.get("data", []):
        rel = item.get("relationships", {}).get("appStoreVersion", {}).get("data") or {}
        if rel.get("id") == VERSION_ID:
            return True
    return False


def main(version_string, build_id):
    step(f"1/7  Point version {VERSION_ID} at {version_string}")
    mutate("PATCH", f"/appStoreVersions/{VERSION_ID}", {
        "data": {"type": "appStoreVersions", "id": VERSION_ID,
                 "attributes": {"versionString": version_string}}})

    step("2/7  Declare export compliance on the build")
    # Without this the build cannot be attached to a version item.
    mutate("PATCH", f"/builds/{build_id}", {
        "data": {"type": "builds", "id": build_id,
                 "attributes": {"usesNonExemptEncryption": False}}})

    step("3/7  Attach the build to the version")
    mutate("PATCH", f"/appStoreVersions/{VERSION_ID}/relationships/build", {
        "data": {"type": "builds", "id": build_id}})

    step("4/7  Cancel the submission holding THIS version (and only that one)")
    # An item-only submission (e.g. a subscription batch) can be active on the
    # same platform at the same time — cancelling by platform alone would kill
    # it. Only a submission whose items include our appStoreVersion is ours.
    subs = call("GET", f"/apps/{APP_ID}/reviewSubmissions",
                params={"filter[platform]": PLATFORM})
    for s in subs.get("data", []):
        state = s["attributes"]["state"]
        if state not in ("UNRESOLVED_ISSUES", "READY_FOR_REVIEW",
                         "WAITING_FOR_REVIEW", "IN_REVIEW"):
            continue
        if not submission_holds_version(s["id"]):
            print(f"  leaving {s['id']} ({state}) alone — does not hold this version")
            continue
        print(f"  cancelling {s['id']} ({state})")
        mutate("PATCH", f"/reviewSubmissions/{s['id']}", {
            "data": {"type": "reviewSubmissions", "id": s["id"],
                     "attributes": {"canceled": True}}})

    step("5/7  Create a fresh review submission")
    sub = mutate("POST", "/reviewSubmissions", {
        "data": {"type": "reviewSubmissions",
                 "relationships": {"app": {"data": {"type": "apps", "id": APP_ID}}},
                 "attributes": {"platform": PLATFORM}}})
    sub_id = sub["data"]["id"] if sub else "<new-submission>"
    print(f"  submission {sub_id}")

    step("6/7  Add the app version to the submission")
    mutate("POST", "/reviewSubmissionItems", {
        "data": {"type": "reviewSubmissionItems",
                 "relationships": {
                     "reviewSubmission": {"data": {"type": "reviewSubmissions", "id": sub_id}},
                     "appStoreVersion": {"data": {"type": "appStoreVersions", "id": VERSION_ID}}}}})
    if DO_SUBMIT:
        items = call("GET", f"/reviewSubmissions/{sub_id}/items")
        print(f"  submission now holds {items['meta']['paging']['total']} item(s)")

    # THIS is what was missing in the 0.6.0 submission and caused the 2.1(b)
    # rejection. An auto-renewable subscription CANNOT be added as a
    # reviewSubmissionItem — the API accepts exactly five relationship names
    # (appStoreVersion, appEvent, appCustomProductPageVersion,
    # appStoreVersionExperiment, appStoreVersionExperimentV2; probed live
    # 2026-07-30). POST /v1/subscriptionSubmissions exists, but for the app's
    # FIRST subscription it refuses with
    # FIRST_SUBSCRIPTION_MUST_BE_SUBMITTED_ON_VERSION — that first one can only
    # be attached in the App Store Connect web UI, together with its
    # subscription GROUP (group page → "Add for review" → pick the draft;
    # verified 2026-08-10, submission 088bcc04).
    step("6b/7  Submit the subscription (separate endpoint — do not skip)")
    if not DO_SUBMIT:
        print("  DRY RUN — would POST /subscriptionSubmissions for "
              f"subscription {SUBSCRIPTION_ID}")
    else:
        sub_state = call("GET", f"/subscriptions/{SUBSCRIPTION_ID}")
        print(f"  subscription state before: "
              f"{sub_state['data']['attributes']['state']}")
        status, resp = asc.call("POST", "/subscriptionSubmissions", {
            "data": {"type": "subscriptionSubmissions",
                     "relationships": {
                         "subscription": {"data": {"type": "subscriptions",
                                                   "id": SUBSCRIPTION_ID}}}}})
        if status not in (200, 201):
            if "FIRST_SUBSCRIPTION_MUST_BE_SUBMITTED_ON_VERSION" in str(resp).upper():
                print("  ASC refused: this is the app's FIRST subscription, and the")
                print("  API cannot attach it to a version submission. Finish in the")
                print("  web UI instead: Subscriptions → subscription group page →")
                print("  'Add for review' → pick this draft submission. That adds the")
                print("  subscription AND its group as review items; then submit there.")
                print("  Submitting the version alone here would repeat the 2.1(b)")
                print("  rejection, so stopping.")
                raise SystemExit(2)
            print(f"  HTTP {status}: {resp}")
            raise SystemExit("aborted at POST /subscriptionSubmissions")
        after = call("GET", f"/subscriptions/{SUBSCRIPTION_ID}")
        print(f"  subscription state after:  "
              f"{after['data']['attributes']['state']}")

    step("7/7  Submit for review")
    if not DO_SUBMIT:
        print("  DRY RUN — no requests were made. Re-run with --submit to execute.")
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
