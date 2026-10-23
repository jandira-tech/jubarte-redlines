import pathlib, sys

path = pathlib.Path("new/src/legacy_doc.rs")
text = path.read_text()

def sub(old, new, count=1):
    global text
    n = text.count(old)
    if n != count:
        print(f"MISMATCH ({n} != {count}): {old[:80]!r}", file=sys.stderr)
        sys.exit(1)
    text = text.replace(old, new)

# --- helpers before main_text ---
sub("""/// The main story's characters, CP 0 to `ccp_text`.
///
/// # Errors
///
/// A piece that points past the `WordDocument` stream: a truncated or
/// corrupt file, refused rather than converted short.
fn main_text(word: &[u8], pieces: &[Piece], ccp_text: u32) -> Result<Vec<StoryChar>> {""",
    """/// The logical characters [`main_text`] may materialize from a
/// WordDocument stream of `bytes` bytes: an honest document never exceeds
/// its stream (a compressed piece is one byte per character), a fast-saved
/// one that reuses physical ranges gets a generous factor, and every
/// smaller stream keeps a floor. A piece table that aliases one short
/// physical range into gigabytes of logical characters is refused before
/// any per-character record exists.
fn logical_char_budget(bytes: usize) -> u64 {
    (bytes as u64).saturating_mul(16).max(1 << 20)
}

/// The refusal for a story whose claimed length no document of its size
/// can hold.
fn huge_story() -> LegacyDocError {
    LegacyDocError::new("the main story claims more characters than its document holds")
}

/// The main story's characters, CP 0 to `ccp_text`.
///
/// # Errors
///
/// A piece that points past the `WordDocument` stream: a truncated or
/// corrupt file, refused rather than converted short; or a story whose
/// claimed character count exceeds what the `WordDocument` stream can
/// hold (a crafted or corrupt piece table).
fn main_text(word: &[u8], pieces: &[Piece], ccp_text: u32) -> Result<Vec<StoryChar>> {""")

# --- ccp_text budget gate ---
sub("""    if covered < ccp_text {
        return Err(LegacyDocError::new(
            "the piece table does not cover the main story",
        ));
    }
    let mut out = Vec::new();""",
    """    if covered < ccp_text {
        return Err(LegacyDocError::new(
            "the piece table does not cover the main story",
        ));
    }
    let budget = logical_char_budget(word.len());
    if u64::from(ccp_text) > budget {
        return Err(huge_story());
    }
    let mut out = Vec::new();""")

# --- cumulative budget during materialization ---
sub("""    for piece in pieces {
        if piece.cp_start >= ccp_text {
            break;
        }
        let end = piece.cp_end.min(ccp_text);
        let width: u32 = if piece.compressed { 1 } else { 2 };""",
    """    let mut logical = 0u64;
    for piece in pieces {
        if piece.cp_start >= ccp_text {
            break;
        }
        let end = piece.cp_end.min(ccp_text);
        // Every logical character is charged to the same budget: a piece
        // table may repeat or overlap physical ranges, and the records it
        // implies are refused before they are materialized.
        logical = logical
            .checked_add(u64::from(end - piece.cp_start))
            .ok_or_else(huge_story)?;
        if logical > budget {
            return Err(huge_story());
        }
        let width: u32 = if piece.compressed { 1 } else { 2 };""")

# --- blocks: O(1) code-frame count ---
sub("""        // Field nesting: true while inside a field's code (before its
        // separator).
        let mut fields: Vec<bool> = Vec::new();""",
    """        // Field nesting: true while inside a field's code (before its
        // separator). `code_frames` counts the frames still in their code
        // phase, so the per-character check below is one comparison, not a
        // scan of every enclosing frame (a crafted story of nested fields
        // made that scan quadratic).
        let mut fields: Vec<bool> = Vec::new();
        let mut code_frames = 0usize;""")
sub("""            // Inside any field's code, even a nested field's result is code.
            let in_code = fields.iter().any(|&code| code);
            let text = match story.ch {
                '\\u{13}' => {
                    fields.push(true);
                    continue;
                }
                '\\u{14}' => {
                    if let Some(top) = fields.last_mut() {
                        *top = false;
                    }
                    continue;
                }
                '\\u{15}' => {
                    fields.pop();
                    continue;
                }""",
    """            // Inside any field's code, even a nested field's result is code.
            let in_code = code_frames > 0;
            let text = match story.ch {
                '\\u{13}' => {
                    fields.push(true);
                    code_frames += 1;
                    continue;
                }
                '\\u{14}' => {
                    if let Some(top) = fields.last_mut() {
                        if *top {
                            code_frames -= 1;
                        }
                        *top = false;
                    }
                    continue;
                }
                '\\u{15}' => {
                    if fields.pop() == Some(true) {
                        code_frames -= 1;
                    }
                    continue;
                }""")

path.write_text(text)
print("legacy_doc.rs edits applied")
