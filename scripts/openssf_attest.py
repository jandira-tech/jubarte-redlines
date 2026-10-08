#!/usr/bin/env python3
# SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
#
# SPDX-License-Identifier: AGPL-3.0-only
"""Review maintainer-only OpenSSF answers interactively; never submit them."""

import argparse
import calendar
import json
from datetime import date
from pathlib import Path

PROJECT = 15267
CRITERIA_URL = "https://www.bestpractices.dev/en/criteria/0"


def month_offset(day, months):
    number = day.year * 12 + day.month - 1 + months
    year, month = divmod(number, 12)
    month += 1
    return date(year, month, min(day.day, calendar.monthrange(year, month)[1]))


def questions(day):
    window = f"{month_offset(day, -12)} through {month_offset(day, -2)}"
    private_window = f"{month_offset(day, -6)} through {day}"
    # id, suggested answer, applicability of N/A, statement, suggested evidence
    return [
        ("know_secure_design", "met", False,
         "I am a primary developer and understand secure-design principles: simplicity, deny-by-default, checking every protected access, open design, separation of privilege, least privilege, minimizing shared mechanisms, usable security, limited attack surface, input validation, and defense in depth. Review the exact OpenSSF details before confirming.",
         "A primary developer confirms knowledge of the secure-design principles required by OpenSSF. See SECURITY.md for the project's threat model."),
        ("know_common_errors", "met", False,
         "A primary developer knows common vulnerabilities relevant to this engine and a mitigation for each: decompression bombs (byte/entry budgets), parser complexity (bounds), path traversal (safe part names), malformed XML/fonts/images (validation, fuzzing, process limits), and external references (no automatic fetching).",
         "A primary developer confirms knowledge of relevant vulnerability classes and their mitigations, documented in SECURITY.md."),
        ("report_responses", "?", False,
         f"Review every bug-report channel for {window}. Was a majority acknowledged? With no reports, explain that explicitly; do not invent a response percentage.", ""),
        ("enhancement_responses", "?", False,
         f"Review enhancement requests for {window}. Was more than half answered? If none were received, state that explicitly.", ""),
        ("vulnerability_report_response", "?", True,
         f"Review private and public vulnerability reports for {private_window}. Did EVERY initial response take at most 14 days? Select N/A only if no reports were received. Save aggregate evidence, not private report contents.", ""),
        ("release_notes_vulns", "?", True,
         "Review engine vulnerabilities with CVE/GHSA or similar assignments known at release time. Were all fixes named in their release notes? N/A is appropriate if no such publicly known engine vulnerabilities existed; dependency advisories alone do not establish an engine CVE.", ""),
        ("vulnerabilities_fixed_60_days", "?", False,
         "Review public advisories and all exceptions. Are there no unpatched medium/higher engine vulnerabilities publicly known for over 60 days? The XML dependency remains affected outside the guarded engine path; assess that exposure before answering.", ""),
        ("vulnerabilities_critical_fixed", "?", False,
         "Review critical vulnerability handling and actual fix dates. Does the project fix critical findings rapidly? A new policy alone does not establish past performance.", ""),
        ("static_analysis_fixed", "?", True,
         "Review static-analysis findings, exploitability decisions and fix dates. Were confirmed medium/higher exploitable findings fixed in a timely way? Explain any N/A claim.", ""),
        ("dynamic_analysis_fixed", "?", True,
         "Review fuzz crashes and other dynamic findings, exploitability decisions and fix dates. Were confirmed medium/higher exploitable findings fixed in a timely way? Explain any N/A claim.", ""),
        ("test_most", "?", False,
         "Review full-suite branch, input-field and functionality coverage. The focused OPC run measured 87.13% lines and 57.61% branches; that does not certify broad project coverage.", ""),
        ("dynamic_analysis", "?", False,
         "Review a major release candidate's actual dynamic-analysis run. Record tool, command, source commit, date and outcome; having fuzz targets is not evidence they ran for that candidate.", ""),
        ("dynamic_analysis_unsafe", "?", True,
         "Own library source forbids unsafe Rust, but the default CLI includes native mimalloc. Confirm routine dynamic analysis with memory-error detection for the relevant shipped unsafe code, or justify N/A for the precise scope assessed.", ""),
        ("crypto_keylength", "?", True,
         "Review shipped TLS/signature settings against the criterion's minimum security strength. Self-update uses TLS; do not treat all crypto as absent. Explain any N/A claim.", ""),
        ("crypto_pfs", "?", True,
         "Confirm the shipped self_update/ureq/rustls key-agreement configuration provides forward secrecy, or justify N/A for the scope assessed.", ""),
        ("crypto_password_storage", "n/a", True,
         "Confirm that the engine does not store passwords to authenticate external users.",
         "The engine does not store passwords for authentication of external users."),
        ("crypto_random", "?", True,
         "Confirm security keys/nonces use a cryptographically secure RNG, including delegated TLS and release-signing mechanisms. The engine's lack of its own key generation does not automatically make delegated randomness N/A.", ""),
    ]


def validate(document, items):
    if document.get("project_id") != PROJECT or document.get("schema_version") != 1:
        raise ValueError("Answer file belongs to another project or schema")
    if not isinstance(document.get("attested_by"), str) or not document["attested_by"].strip():
        raise ValueError("Answer file must identify the attesting maintainer")
    if not isinstance(document.get("answers"), dict):
        raise ValueError("Answers must be an object")
    allowed = {item[0]: item[2] for item in items}
    for key, answer in document["answers"].items():
        if key not in allowed or not isinstance(answer, dict):
            raise ValueError("Unknown or invalid criterion: " + key)
        status = answer.get("status")
        if status not in ("met", "unmet", "?", "n/a") or (status == "n/a" and not allowed[key]):
            raise ValueError("Invalid status for " + key)
        if not isinstance(answer.get("justification"), str):
            raise ValueError("Missing justification for " + key)
        if status != "?" and not answer["justification"].strip():
            raise ValueError("Empty justification for " + key)
    return document


def walk(document, items, read, emit, save, review=False):
    """Interaction with injected I/O; no filesystem, network or clock access."""
    for index, (key, default, allow_na, statement, suggested) in enumerate(items, 1):
        previous = document["answers"].get(key)
        if previous and not review:
            continue
        if previous:
            default, suggested = previous["status"], previous["justification"]
        emit(f"\n[{index}/{len(items)}] {key}\n{statement}")
        emit("Enter accepts the displayed default as your answer. '?' means unknown.")
        choices = "met/unmet/" + ("n/a/" if allow_na else "") + "?/skip/quit"
        while True:
            status = read(f"Answer ({choices}) [{default}]: ").strip().lower() or default
            if status == "quit":
                return
            if status == "skip":
                break
            if status not in ("met", "unmet", "?", "n/a") or (status == "n/a" and not allow_na):
                emit("Choose one of the displayed answers.")
                continue
            rationale = suggested if status == default else ""
            while True:
                prompt = f"Justification [{rationale}]: " if rationale else "Justification (no private details): "
                justification = read(prompt).strip() or rationale
                if justification or status == "?":
                    break
                emit("Provide evidence or an applicability explanation; '?' can remain blank.")
            document["answers"][key] = {"status": status, "justification": justification}
            save(document)
            break


def summary(document):
    lines = [f"OpenSSF project {PROJECT}; attested by {document['attested_by']}; review date {document['review_date']}"]
    for key, answer in document["answers"].items():
        lines.append(f"\n{key}: {answer['status']}\n{answer['justification']}")
    return "\n".join(lines)


def save_file(path, document):
    path.parent.mkdir(parents=True, exist_ok=True)
    temporary = path.with_name(path.name + ".tmp")
    temporary.write_text(json.dumps(document, indent=2) + "\n", encoding="utf-8")
    temporary.replace(path)


def main(argv=None, read=input, emit=print):
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--output", type=Path, default=Path(__file__).resolve().parents[1] / "out/openssf-attestations.json")
    parser.add_argument("--name", help="Name of the maintainer personally confirming the answers")
    parser.add_argument("--date", type=date.fromisoformat, default=date.today(), help="Assessment date, YYYY-MM-DD")
    parser.add_argument("--review", action="store_true", help="Revisit saved answers rather than skipping them")
    parser.add_argument("--list", action="store_true", help="Show questions/defaults without answering or saving")
    parser.add_argument("--show", action="store_true", help="Print saved answers for copying to the badge form")
    args = parser.parse_args(argv)
    items = questions(args.date)
    if args.list:
        for key, default, _, statement, _ in items:
            emit(f"{key} [{default}]: {statement}")
        return 0
    try:
        if args.output.exists():
            document = validate(json.loads(args.output.read_text(encoding="utf-8")), items)
            if args.name and args.name != document["attested_by"]:
                raise ValueError("Use a separate --output file for a different maintainer")
        elif args.show:
            raise ValueError("No saved answers yet")
        else:
            name = (args.name or read("Your name (attesting maintainer): ")).strip()
            if not name:
                raise ValueError("An attesting maintainer name is required")
            document = {"schema_version": 1, "project_id": PROJECT, "attested_by": name,
                        "review_date": args.date.isoformat(), "answers": {}}
        if args.show:
            emit(summary(document))
            return 0
        if document["review_date"] != args.date.isoformat():
            raise ValueError("Saved assessment date differs; pass its --date to resume or use a fresh --output")
        emit(f"OpenSSF project {PROJECT}: {CRITERIA_URL}\nDefaults are suggestions, not verified facts.\nAnswers stay local; nothing is submitted. Quit or Ctrl-C to stop; saved answers resume next time.")
        walk(document, items, read, emit, lambda value: save_file(args.output, value), args.review)
        emit(f"Answer file: {args.output} (written after each completed item). Use --show to print or --review to revisit answers.")
        return 0
    except (EOFError, KeyboardInterrupt):
        emit("\nStopped. Previously completed answers are preserved.")
        return 0
    except (ValueError, OSError) as error:
        emit("Error: " + str(error))
        return 1


if __name__ == "__main__":
    raise SystemExit(main())
