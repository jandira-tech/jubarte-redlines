#!/usr/bin/env python3
# SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
#
# SPDX-License-Identifier: AGPL-3.0-only

"""Pure interaction tests plus explicitly local filesystem integration tests."""
import copy
import json
import tempfile
import unittest
from datetime import date
from pathlib import Path

import openssf_attest as app

DAY = date(2026, 10, 7)


def reader(*answers):
    iterator = iter(answers)
    def read(_):
        try:
            value = next(iterator)
        except StopIteration:
            raise EOFError from None
        if isinstance(value, BaseException):
            raise value
        return value
    return read


def document():
    return dict(schema_version=1, project_id=15267, attested_by="Maintainer",
                review_date=DAY.isoformat(), answers={})


class Interaction(unittest.TestCase):
    def test_month_boundaries_and_windows(self):
        self.assertEqual(app.month_offset(date(2024, 3, 31), -1), date(2024, 2, 29))
        items = app.questions(DAY)
        self.assertEqual(len({i[0] for i in items}), len(items))
        self.assertIn("2025-10-07 through 2026-08-07", items[2][3])
        self.assertTrue(all(i[1] == "?" for i in items if "response" in i[0]))

    def test_defaults_require_interactive_acceptance_and_save_each_item(self):
        doc, saved = document(), []
        app.walk(doc, app.questions(DAY)[:2], reader("", "", "unmet", "Need training"),
                 lambda _: None, lambda d: saved.append(copy.deepcopy(d)))
        self.assertEqual(len(saved), 2)
        self.assertEqual(len(saved[0]["answers"]), 1)
        self.assertEqual(doc["answers"]["know_secure_design"]["status"], "met")
        self.assertEqual(doc["answers"]["know_common_errors"]["status"], "unmet")
        self.assertIs(app.validate(doc, app.questions(DAY)), doc)
        self.assertIn("Need training", app.summary(doc))

    def test_skip_quit_unknown_and_invalid_answers(self):
        doc, output = document(), []
        items = app.questions(DAY)
        app.walk(doc, items[:4], reader("n/a", "bad", "?", "", "skip", "quit"), output.append, lambda _: None)
        self.assertEqual(list(doc["answers"]), ["know_secure_design"])
        self.assertEqual(doc["answers"]["know_secure_design"]["status"], "?")
        self.assertEqual(output.count("Choose one of the displayed answers."), 2)

    def test_evidence_required_and_resume_review(self):
        doc = document()
        app.walk(doc, app.questions(DAY)[2:3], reader("met", "", "Reviewed all channels: no reports"), lambda _: None, lambda _: None)
        app.walk(doc, app.questions(DAY)[2:3], reader(), lambda _: None, lambda _: self.fail("resaved"))
        app.walk(doc, app.questions(DAY)[2:3], reader("", ""), lambda _: None, lambda _: None, review=True)
        self.assertIn("no reports", doc["answers"]["report_responses"]["justification"])

    def test_validation_rejects_invalid_saved_attestations(self):
        for changes in [dict(project_id=0), dict(schema_version=0), dict(attested_by=""), dict(answers=[]),
                        dict(answers={"invalid": {}}), dict(answers={"know_secure_design": []}),
                        dict(answers={"know_secure_design": dict(status="n/a", justification="reason")}),
                        dict(answers={"know_secure_design": dict(status="bogus", justification="reason")}),
                        dict(answers={"know_secure_design": dict(status="met", justification=0)}),
                        dict(answers={"know_secure_design": dict(status="met", justification="")})]:
            with self.subTest(changes=changes), self.assertRaises(ValueError):
                app.validate(dict(document(), **changes), app.questions(DAY))


class FilesystemIntegration(unittest.TestCase):
    """Real local persistence is intentional integration coverage, never network."""
    def test_cli_persistence_resume_show_and_interrupt(self):
        with tempfile.TemporaryDirectory() as folder:
            path = Path(folder) / "answers.json"
            argv = ["--output", str(path), "--date", DAY.isoformat()]
            output = []
            self.assertEqual(app.main(argv + ["--list"], emit=output.append), 0)
            self.assertFalse(path.exists())
            self.assertEqual(app.main(argv + ["--show"], emit=output.append), 1)
            self.assertEqual(app.main(argv, reader(""), output.append), 1)
            self.assertEqual(app.main(argv, reader("Maintainer", "", "", KeyboardInterrupt()), output.append), 0)
            self.assertEqual(json.loads(path.read_text())["answers"]["know_secure_design"]["status"], "met")
            self.assertEqual(app.main(argv + ["--show"], emit=output.append), 0)
            self.assertEqual(app.main(argv, reader("quit"), output.append), 0)
            self.assertEqual(app.main(argv + ["--name", "Other"], emit=output.append), 1)
            self.assertEqual(app.main(argv + ["--date", "2026-10-08"], emit=output.append), 1)
            self.assertEqual(app.main(argv + ["--review"], reader("?", "", "quit"), output.append), 0)
            path.write_text("bad json")
            self.assertEqual(app.main(argv, emit=output.append), 1)
            path.unlink()
            self.assertEqual(app.main(argv + ["--name", "Maintainer"], reader("quit"), output.append), 0)
            self.assertFalse(path.exists())
            path.mkdir()
            self.assertEqual(app.main(argv, emit=output.append), 1)
            self.assertIn("Stopped", "\n".join(output))


if __name__ == "__main__":
    unittest.main()
