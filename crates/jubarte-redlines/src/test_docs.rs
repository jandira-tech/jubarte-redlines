// SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
//
// SPDX-License-Identifier: AGPL-3.0-only

//! Document builders shared by the comparer's sweep tests: WordprocessingML
//! paragraphs, a seeded generator, and the template shapes the sweeps have
//! in common.

/// `text` escaped for an XML text node.
pub(crate) fn esc(text: &str) -> String {
    text.replace('&', "&amp;").replace('<', "&lt;")
}

/// One paragraph: optional style, optional list level, run properties.
pub(crate) fn para(text: &str, style: &str, level: Option<u8>, rpr: &str) -> String {
    let style = if style.is_empty() {
        String::new()
    } else {
        format!("<w:pStyle w:val='{style}'/>")
    };
    let num = level.map_or_else(String::new, |l| {
        format!("<w:numPr><w:ilvl w:val='{l}'/><w:numId w:val='9'/></w:numPr>")
    });
    let run = if text.is_empty() {
        String::new()
    } else {
        format!(
            "<w:r><w:rPr>{rpr}</w:rPr><w:t xml:space='preserve'>{}</w:t></w:r>",
            esc(text)
        )
    };
    format!("<w:p><w:pPr>{style}{num}</w:pPr>{run}</w:p>")
}

/// A paragraph of plain text.
pub(crate) fn plain(text: &str) -> String {
    para(text, "", None, "")
}

/// A deterministic linear congruential generator.
pub(crate) struct Lcg(pub(crate) u64);

impl Lcg {
    pub(crate) fn next(&mut self) -> u64 {
        self.0 = self
            .0
            .wrapping_mul(6_364_136_223_846_793_005)
            .wrapping_add(1_442_695_040_888_963_407);
        self.0 >> 33
    }

    /// A number in `0..n`.
    pub(crate) fn below(&mut self, n: usize) -> usize {
        (self.next() % n as u64) as usize
    }

    /// True `percent` times in a hundred.
    pub(crate) fn chance(&mut self, percent: u64) -> bool {
        self.next() % 100 < percent
    }
}

/// `n` words drawn from `pool`, joined by spaces.
pub(crate) fn words(rng: &mut Lcg, pool: &[&str], n: usize) -> String {
    (0..n)
        .map(|_| pool[rng.below(pool.len())])
        .collect::<Vec<_>>()
        .join(" ")
}

/// The words of a whitespace-separated list, in order.
pub(crate) fn split(list: &str) -> Vec<&str> {
    list.split_whitespace().collect()
}

/// A template generator over one word pool.
pub(crate) struct Shapes<'a> {
    pub(crate) pool: &'a [&'a str],
    /// A filler paragraph holds `3 + below(filler_span)` words.
    pub(crate) filler_span: usize,
}

impl Shapes<'_> {
    /// One word of the pool.
    pub(crate) fn pick(&self, rng: &mut Lcg) -> String {
        self.pool[rng.below(self.pool.len())].to_string()
    }

    /// `n` paragraphs of pool words.
    pub(crate) fn filler(&self, rng: &mut Lcg, n: usize) -> Vec<String> {
        (0..n)
            .map(|_| {
                let k = 3 + rng.below(self.filler_span);
                plain(&words(rng, self.pool, k))
            })
            .collect()
    }

    /// A memorandum heading block, then `base + below(span)` paragraphs.
    pub(crate) fn memo(&self, rng: &mut Lcg, base: usize, span: usize) -> Vec<String> {
        let mut out = vec![
            para("MEMORANDUM", "Title", None, ""),
            plain(&format!("TO: {}", self.pick(rng))),
            plain(&format!("FROM: {}", self.pick(rng))),
            plain("DATE: March 3, 2040"),
            plain(&format!("RE: {}", self.pick(rng))),
            plain("Dear colleagues,"),
        ];
        let n = base + rng.below(span);
        out.extend(self.filler(rng, n));
        out
    }

    /// A file-name line, a title, then `base + below(span)` paragraphs.
    pub(crate) fn essay(&self, rng: &mut Lcg, base: usize, span: usize) -> Vec<String> {
        let mut out = vec![plain(&format!("file_{}.docx", 190 + rng.below(9)))];
        out.push(para(
            &format!("{} images essay", self.pick(rng)),
            "Title",
            None,
            "",
        ));
        let n = base + rng.below(span);
        out.extend(self.filler(rng, n));
        out
    }

    /// A demo title and two "This document demonstrates" lines.
    pub(crate) fn demo(&self, rng: &mut Lcg) -> Vec<String> {
        let title = format!("{} {} Demo", self.pick(rng), self.pick(rng));
        let line = format!(
            "This document demonstrates {} {}",
            self.pick(rng),
            self.pick(rng)
        );
        vec![
            para(&title, "Title", None, ""),
            plain(&line),
            plain(&format!("This text is {}", self.pick(rng))),
        ]
    }

    /// A list title and three numbered items.
    pub(crate) fn list_demo(&self, rng: &mut Lcg) -> Vec<String> {
        let mut out = vec![para(
            &format!("{} List Demo", self.pick(rng)),
            "Title",
            None,
            "",
        )];
        for ordinal in ["First", "Second", "Third"] {
            out.push(para(&format!("{ordinal} item"), "", Some(0), ""));
        }
        out
    }

    /// Four lettered section lines, each followed by one paragraph.
    pub(crate) fn lettered(&self, rng: &mut Lcg) -> Vec<String> {
        let mut out = Vec::new();
        for (i, letter) in ['A', 'B', 'C', 'D'].iter().enumerate() {
            out.push(plain(&format!("{letter}) {} section {i}", self.pick(rng))));
            out.extend(self.filler(rng, 1));
        }
        out
    }
}
