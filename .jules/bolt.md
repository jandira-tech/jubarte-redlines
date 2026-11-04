## 2026-11-04 - [Avoid eager String/Vec allocations for Document AST]
**Learning:** Eagerly mapping document AST nodes (like Block::Paragraph and Block::Table) into a Vec<String> using .flatten().cloned().collect() creates significant O(N) memory overhead for large documents.
**Action:** Use lazy evaluation with iterators (e.g., .any(), .find()) directly against the AST elements, avoiding unnecessary allocations of intermediate collections or giant concatenated strings.
