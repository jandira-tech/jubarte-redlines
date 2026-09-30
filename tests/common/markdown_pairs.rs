// SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
//
// SPDX-License-Identifier: AGPL-3.0-only

//! Pairs of Markdown documents the diff suites share.

/// Pairs of documents, old then new.
pub const PAIRS: &[(&str, &str, &str)] = &[
    (
        "words",
        "The fee is ten dollars.\n",
        "The fee is twelve dollars a month.\n",
    ),
    (
        "phrase",
        "The quick brown fox jumps over the lazy dog.\n",
        "The slow red fox walks around the lazy dog.\n",
    ),
    (
        "unrelated",
        "Lorem ipsum dolor sit amet.\n",
        "Completely different words.\n",
    ),
    ("append paragraph", "A.\n\nB.\n", "A.\n\nB.\n\nC.\n"),
    ("prepend paragraph", "B.\n\nC.\n", "A.\n\nB.\n\nC.\n"),
    ("insert paragraph", "A.\n\nC.\n", "A.\n\nB.\n\nC.\n"),
    ("remove paragraph", "A.\n\nB.\n\nC.\n", "A.\n\nC.\n"),
    ("remove first", "A.\n\nB.\n", "B.\n"),
    ("remove last", "A.\n\nB.\n", "A.\n"),
    (
        "split paragraph",
        "One two. Three four.\n",
        "One two.\n\nThree four.\n",
    ),
    (
        "merge paragraphs",
        "One two.\n\nThree four.\n",
        "One two. Three four.\n",
    ),
    (
        "line inside paragraph",
        "first\nthird\n",
        "first\nsecond\nthird\n",
    ),
    (
        "line out of paragraph",
        "first\nsecond\nthird\n",
        "first\nthird\n",
    ),
    ("first line out", "x\ny\n", "y\n"),
    ("list item added", "- a\n- c\n", "- a\n- b\n- c\n"),
    ("list item removed", "- a\n- b\n- c\n", "- a\n- c\n"),
    (
        "list item edited",
        "1. alpha beta\n2. gamma\n",
        "1. alpha delta\n2. gamma\n",
    ),
    (
        "nested list",
        "- a\n  - x\n- b\n",
        "- a\n  - x\n  - y\n- b\n",
    ),
    (
        "heading edited",
        "# Intro\n\nText.\n",
        "# Introduction and scope\n\nText.\n",
    ),
    ("heading level", "# Title\n\nText.\n", "## Title\n\nText.\n"),
    ("heading added", "Text.\n", "# Title\n\nText.\n"),
    (
        "quote",
        "> quoted words here\n",
        "> quoted other words here\n",
    ),
    (
        "table cell",
        "| a | b |\n|---|---|\n| 1 | 2 |\n",
        "| a | b |\n|---|---|\n| 1 | 3 |\n",
    ),
    (
        "table row added",
        "| a | b |\n|---|---|\n| 1 | 2 |\n",
        "| a | b |\n|---|---|\n| 1 | 2 |\n| 3 | 4 |\n",
    ),
    (
        "code block",
        "```\nlet x = 1;\n```\n",
        "```\nlet x = 2;\n```\n",
    ),
    ("emphasis added", "a plain word\n", "a **bold** word\n"),
    (
        "link changed",
        "See [the site](https://a.example).\n",
        "See [the new site](https://b.example).\n",
    ),
    (
        "footnote",
        "Text.[^1]\n\n[^1]: Old note.\n",
        "Text.[^1]\n\n[^1]: New note.\n",
    ),
    ("task", "- [ ] write\n", "- [x] write\n"),
    ("to empty", "Some text.\n", ""),
    ("from empty", "", "Some text.\n"),
    ("delimiters in text", "a {++b++} c\n", "a {++d++} c\n"),
    (
        "renumbered list",
        "1. a\n2. b\n3. c\n",
        "1. a\n2. new\n3. b\n4. c\n",
    ),
    (
        "list after paragraph",
        "Intro.\n",
        "Intro.\n\n- one\n- two\n",
    ),
    (
        "list removed",
        "Intro.\n\n- one\n- two\n\nEnd.\n",
        "Intro.\n\nEnd.\n",
    ),
    (
        "heading removed",
        "# A\n\nText.\n\n# B\n\nMore.\n",
        "# A\n\nText.\n\nMore.\n",
    ),
    (
        "setext heading",
        "Title\n=====\n\nText.\n",
        "Better title\n=====\n\nText.\n",
    ),
    ("code line added", "```\na\nb\n```\n", "```\na\nx\nb\n```\n"),
    ("nested quote", "> > deep words\n", "> > deeper words\n"),
    ("note added", "Text.\n", "Text.[^n]\n\n[^n]: A new note.\n"),
    (
        "many edits",
        "one two three four five six\n",
        "one 2 three 4 five 6\n",
    ),
    ("whitespace only", "a  b\n", "a b\n"),
    ("reordered", "A.\n\nB.\n\nC.\n", "C.\n\nA.\n\nB.\n"),
    (
        "last replaced by heading",
        "Intro.\n\nOld end.\n",
        "Intro.\n\n# New end\n",
    ),
    (
        "first replaced by list",
        "Old start.\n\nRest.\n",
        "- new start\n\nRest.\n",
    ),
    (
        "table added",
        "Intro.\n",
        "Intro.\n\n| a | b |\n|---|---|\n| 1 | 2 |\n",
    ),
    (
        "table removed",
        "Intro.\n\n| a | b |\n|---|---|\n| 1 | 2 |\n\nEnd.\n",
        "Intro.\n\nEnd.\n",
    ),
    (
        "mixed",
        "# Terms\n\nPayment is due in 30 days.\n\n- Delivery\n- Warranty\n\nSigned.\n",
        "# Terms of sale\n\nPayment is due in 45 days.\n\n- Delivery\n- Returns\n- Warranty\n\nLate fees apply.\n\nSigned.\n",
    ),
];
