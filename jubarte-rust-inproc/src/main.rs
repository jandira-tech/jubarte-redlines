// SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
//
// SPDX-License-Identifier: AGPL-3.0-only

//! Long-lived `compare_documents` worker for the neurotic_docx_bench speed harness.
//!
//! Line-oriented stdin → stdout protocol (identical to `docxodus-csharp-inproc`):
//!
//! ```text
//! READY
//! COMPARE <basePath> <nextPath> <outPath>
//!   → OK <bytes> <ms>
//!   → ERR <message>
//! QUIT
//!   → BYE
//! ```
//!
//! Paths may be space-separated (temp paths without spaces) or tab-separated
//! (spaces in paths). One process handles many compares so timing isolates the
//! algorithm from CLI spawn + I/O tax.

use std::io::{self, BufRead, Write};
use std::path::Path;
use std::time::Instant;

#[global_allocator]
static GLOBAL: mimalloc::MiMalloc = mimalloc::MiMalloc;

fn main() {
    let stdin = io::stdin();
    let mut stdout = io::stdout();

    writeln!(stdout, "READY").expect("write READY");
    stdout.flush().expect("flush READY");

    for line in stdin.lock().lines() {
        let Ok(line) = line else { break };
        let line = line.trim();
        if line.is_empty() {
            continue;
        }
        if line == "QUIT" {
            let _ = writeln!(stdout, "BYE");
            let _ = stdout.flush();
            break;
        }
        if !line.starts_with("COMPARE ") {
            let _ = writeln!(stdout, "ERR unknown command");
            let _ = stdout.flush();
            continue;
        }
        let rest = &line["COMPARE ".len()..];
        let paths = parse_three_paths(rest);
        let Some((base_path, next_path, out_path)) = paths else {
            let _ = writeln!(stdout, "ERR expected 3 paths");
            let _ = stdout.flush();
            continue;
        };
        match compare_one(base_path, next_path, out_path) {
            Ok((nbytes, ms)) => {
                let _ = writeln!(stdout, "OK {nbytes} {ms:.3}");
            }
            Err(msg) => {
                let clean = msg.replace('\n', " ").replace('\r', " ");
                let _ = writeln!(stdout, "ERR {clean}");
            }
        }
        let _ = stdout.flush();
    }
}

fn parse_three_paths(rest: &str) -> Option<(&str, &str, &str)> {
    if rest.contains('\t') {
        let parts: Vec<&str> = rest.split('\t').collect();
        if parts.len() != 3 {
            return None;
        }
        return Some((parts[0], parts[1], parts[2]));
    }
    // Space-separated; temp harness paths never contain spaces.
    let mut parts = rest.splitn(3, ' ');
    let a = parts.next()?.trim();
    let b = parts.next()?.trim();
    let c = parts.next()?.trim();
    if a.is_empty() || b.is_empty() || c.is_empty() {
        return None;
    }
    Some((a, b, c))
}

fn compare_one(base: &str, next: &str, out: &str) -> Result<(usize, f64), String> {
    let left = std::fs::read(Path::new(base)).map_err(|e| format!("read base: {e}"))?;
    let right = std::fs::read(Path::new(next)).map_err(|e| format!("read next: {e}"))?;
    let t0 = Instant::now();
    let result = jubarte::document_comparer::compare_documents(&left, &right, "Redline")
        .map_err(|e| format!("{e}"))?;
    let ms = t0.elapsed().as_secs_f64() * 1000.0;
    std::fs::write(Path::new(out), &result).map_err(|e| format!("write out: {e}"))?;
    Ok((result.len(), ms))
}
