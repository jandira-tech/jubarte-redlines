"""Contract tests for asc-resubmit.py against a recorded fake ASC API.

Run with:
    uv run --with pytest,cryptography pytest scripts/test_asc_resubmit.py -q

Covers CodeRabbit #3691882450 (dry run must not mutate), #3691882458 (cancel
only submissions that actually hold this version), and #3691882467 (abort with
guidance when the API refuses the first-subscription flow) on PR #1.
"""

import importlib.util
import json
from pathlib import Path

import pytest

MOD_PATH = Path(__file__).resolve().parent / "asc-resubmit.py"


def load_script():
    spec = importlib.util.spec_from_file_location("asc_resubmit_under_test", MOD_PATH)
    if spec is None or spec.loader is None:
        raise RuntimeError(f"cannot load {MOD_PATH}")
    mod = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(mod)
    return mod


class FakeAsc:
    """Records every request; serves canned responses keyed by (method, path)."""

    def __init__(self, responses):
        self.responses = responses
        self.requests = []

    def call(self, method, path, body=None, params=None):
        self.requests.append((method, path, body))
        for (m, prefix), resp in self.responses.items():
            if method == m and path.startswith(prefix):
                return resp if isinstance(resp, tuple) else (200, resp)
        if method in ("PATCH", "POST"):
            return 200, {"data": {"id": "generated", "attributes": {"state": "OK"}}}
        return 200, {"data": []}

    def writes(self):
        return [(m, p) for (m, p, _b) in self.requests if m in ("PATCH", "POST", "DELETE")]


def submission(sub_id, state="READY_FOR_REVIEW"):
    return {"id": sub_id, "attributes": {"state": state}}


def items_holding(version_id):
    return {
        "data": [
            {
                "type": "reviewSubmissionItems",
                "relationships": {
                    "appStoreVersion": {"data": {"type": "appStoreVersions", "id": version_id}}
                },
            }
        ],
        "meta": {"paging": {"total": 1}},
    }


def test_dry_run_performs_zero_mutations(capsys):
    mod = load_script()
    fake = FakeAsc(
        {
            ("GET", f"/apps/{mod.APP_ID}/reviewSubmissions"): {
                "data": [submission("SUB-A")],
                "meta": {"paging": {"total": 1}},
            },
            ("GET", "/reviewSubmissions/SUB-A/items"): items_holding(mod.VERSION_ID),
        }
    )
    mod.asc = fake
    mod.DO_SUBMIT = False

    mod.main("9.9.9", "BUILD-1")

    assert fake.writes() == [], (
        "dry run must not PATCH/POST anything, got: " + json.dumps(fake.writes())
    )


def test_submit_cancels_only_submissions_holding_this_version():
    mod = load_script()
    fake = FakeAsc(
        {
            ("GET", f"/apps/{mod.APP_ID}/reviewSubmissions"): {
                "data": [submission("SUB-OURS"), submission("SUB-OTHER")],
                "meta": {"paging": {"total": 2}},
            },
            ("GET", "/reviewSubmissions/SUB-OURS/items"): items_holding(mod.VERSION_ID),
            ("GET", "/reviewSubmissions/SUB-OTHER/items"): items_holding("some-other-version"),
            ("POST", "/reviewSubmissions"): (201, {"data": {"id": "SUB-NEW"}}),
            ("GET", "/reviewSubmissions/SUB-NEW/items"): items_holding(mod.VERSION_ID),
            ("GET", f"/subscriptions/{mod.SUBSCRIPTION_ID}"): {
                "data": {"attributes": {"state": "READY_TO_SUBMIT"}}
            },
            ("POST", "/subscriptionSubmissions"): (
                201,
                {"data": {"id": "SS-1", "attributes": {"state": "WAITING_FOR_REVIEW"}}},
            ),
            ("GET", "/reviewSubmissions/SUB-NEW"): {
                "data": {"id": "SUB-NEW", "attributes": {"state": "WAITING_FOR_REVIEW"}}
            },
        }
    )
    mod.asc = fake
    mod.DO_SUBMIT = True

    mod.main("9.9.9", "BUILD-1")

    cancelled = [
        p
        for (m, p, b) in fake.requests
        if m == "PATCH" and b and b.get("data", {}).get("attributes", {}).get("canceled")
    ]
    assert cancelled == ["/reviewSubmissions/SUB-OURS"], cancelled


def test_first_subscription_refusal_aborts_before_final_submit(capsys):
    mod = load_script()
    fake = FakeAsc(
        {
            ("GET", f"/apps/{mod.APP_ID}/reviewSubmissions"): {
                "data": [],
                "meta": {"paging": {"total": 0}},
            },
            ("POST", "/reviewSubmissions"): (201, {"data": {"id": "SUB-NEW"}}),
            ("GET", "/reviewSubmissions/SUB-NEW/items"): items_holding(mod.VERSION_ID),
            ("GET", f"/subscriptions/{mod.SUBSCRIPTION_ID}"): {
                "data": {"attributes": {"state": "READY_TO_SUBMIT"}}
            },
            ("POST", "/subscriptionSubmissions"): (
                409,
                {
                    "errors": [
                        {
                            "code": "STATE_ERROR.FIRST_SUBSCRIPTION_MUST_BE_SUBMITTED_ON_VERSION",
                            "detail": "first subscription must be submitted on a version",
                        }
                    ]
                },
            ),
        }
    )
    mod.asc = fake
    mod.DO_SUBMIT = True

    with pytest.raises(SystemExit):
        mod.main("9.9.9", "BUILD-1")

    final_submits = [
        (m, p, b)
        for (m, p, b) in fake.requests
        if m == "PATCH" and p == "/reviewSubmissions/SUB-NEW"
    ]
    assert final_submits == [], "must not submit a version-only review submission (2.1(b))"
    out = capsys.readouterr().out
    assert "group" in out.lower(), "abort message must point at the web-UI group flow"
