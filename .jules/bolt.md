## 2024-03-24 - [Avoid Iterating Over Entire Story for Paragraph Local Context]
**Learning:** In `jubarte-redlines` markdown writing, `self.story()` returns the entire document or footnote item stream up to the current point. Iterating over `self.story()` per-item or per-comment introduces O(N^2) performance regressions. Always scope backward scans to the current paragraph using `self.paragraph.unwrap_or(0)`.
**Action:** When finding context for an item in a paragraph, start from the paragraph's start index instead of iterating from the end of the entire story.
