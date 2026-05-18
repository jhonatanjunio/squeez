use crate::commands::Handler;
use crate::config::Config;
use crate::strategies::{grouping, smart_filter, truncation};

pub struct FsHandler;

const NOISY_ENV_PREFIXES: &[&str] = &[
    "PATH=",
    "LS_COLORS=",
    "TERM=",
    "PS1=",
    "PROMPT=",
    "MANPATH=",
];

// Viewer commands that may display code files.
const VIEWER_CMDS: &[&str] = &["cat", "head", "tail", "less", "more", "bat"];

/// Returns (file_path, language_tag) if `cmd` is a viewer command targeting a code file.
fn extract_path_and_ext(cmd: &str) -> Option<(String, &'static str)> {
    let mut parts = cmd.split_whitespace();
    let first = parts.next()?;
    let base = first.rsplit('/').next().unwrap_or(first);
    if !VIEWER_CMDS.contains(&base) {
        return None;
    }

    // Collect remaining tokens, skipping numeric flags like -n 50.
    let mut path: Option<String> = None;
    let mut skip_next = false;
    for tok in parts {
        if skip_next {
            skip_next = false;
            continue;
        }
        if tok == "-n" || tok == "-c" {
            skip_next = true;
            continue;
        }
        // Skip flag-only tokens (e.g. -f, --follow, -q, --plain).
        if tok.starts_with('-') {
            // But if the flag embeds a value like -n50, nothing to skip.
            continue;
        }
        path = Some(tok.to_string());
        break;
    }

    let path = path?;
    let ext = std::path::Path::new(&path)
        .extension()
        .and_then(|e| e.to_str())?;

    let lang: &'static str = match ext {
        "rs" => "rust",
        "py" | "pyw" => "python",
        "ts" | "tsx" | "js" | "jsx" | "mjs" | "cjs" => "ts",
        "go" => "go",
        "java" => "java",
        "rb" => "ruby",
        "kt" | "kts" => "kotlin",
        "swift" => "swift",
        "c" | "cc" | "cpp" | "cxx" | "h" | "hpp" | "hxx" => "c",
        _ => return None,
    };

    Some((path, lang))
}

/// Returns true if the line looks like an indented signature — i.e. a
/// signature line inside an `impl` block (Rust), a class body (Python,
/// TypeScript, Java, Kotlin, Swift, Ruby), or any nested scope. Without
/// this, signature-mode silently drops every method on a large source file
/// because top-level matching only sees the enclosing `impl`/`class` line.
fn is_indented_signature(line: &str, lang: &str) -> bool {
    let trimmed = line.trim_start();
    if trimmed.len() == line.len() {
        // Not indented at all — let `is_signature` handle the top-level case.
        return false;
    }
    if trimmed.is_empty() {
        return false;
    }
    match lang {
        "ts" => {
            // Class-member shapes (positive list — these never appear at file
            // top level, so they're safe to match only here).
            const TS_MEMBER_PREFIXES: &[&str] = &[
                "public ",
                "private ",
                "protected ",
                "static ",
                "async ",
                "get ",
                "set ",
                "readonly ",
                "abstract ",
                "override ",
            ];
            if TS_MEMBER_PREFIXES.iter().any(|p| trimmed.starts_with(p)) {
                return true;
            }
            // Body-content veto: these prefixes are valid at the top level
            // (`export const`, `type Foo = ...`) but inside a method body
            // they're just local-variable declarations.
            const BODY_PREFIXES: &[&str] = &[
                "const ",
                "let ",
                "var ",
                "type ",
                "interface ",
                "enum ",
            ];
            if BODY_PREFIXES.iter().any(|p| trimmed.starts_with(p)) {
                return false;
            }
        }
        "java" | "kotlin" => {
            // Same idea: class-member visibility/modifier keywords are
            // already in the top-level matcher; keep that working when
            // indented too. Nothing extra to do.
        }
        _ => {}
    }
    is_signature(trimmed, lang)
}

/// Returns true if the line is a doc-comment, decorator, or attribute that
/// belongs to the *next* declaration. Signature-mode preserves the most
/// recent such run immediately above each kept signature, because they
/// carry intent, derives, decorators, and overload hints — exactly what
/// the LLM needs to avoid re-reading the file.
fn is_doc_or_attribute(line: &str, lang: &str) -> bool {
    let trimmed = line.trim_start();
    if trimmed.is_empty() {
        return false;
    }
    match lang {
        "rust" => {
            trimmed.starts_with("///")
                || trimmed.starts_with("//!")
                || trimmed.starts_with("#[")
                || trimmed.starts_with("#![")
        }
        "python" => {
            trimmed.starts_with("@")
                || trimmed.starts_with("#")
                || trimmed.starts_with("\"\"\"")
                || trimmed.starts_with("'''")
        }
        "ts" | "java" | "kotlin" | "c" => {
            trimmed.starts_with("//")
                || trimmed.starts_with("/*")
                || trimmed.starts_with("*")
                || trimmed.starts_with("@")
        }
        "go" => trimmed.starts_with("//"),
        "ruby" => trimmed.starts_with("#"),
        "swift" => trimmed.starts_with("///") || trimmed.starts_with("//"),
        _ => false,
    }
}

/// Returns true if the line looks like a signature for the given language.
fn is_signature(line: &str, lang: &str) -> bool {
    match lang {
        "rust" => {
            let prefixes: &[&str] = &[
                "pub fn ",
                "fn ",
                "async fn ",
                "pub async fn ",
                "unsafe fn ",
                "pub unsafe fn ",
                "impl ",
                "trait ",
                "struct ",
                "enum ",
                "pub mod ",
                "pub use ",
                "pub struct ",
                "pub enum ",
                "pub trait ",
                "pub type ",
                "pub const ",
                "pub impl ",
                "pub trait ",
            ];
            prefixes.iter().any(|p| line.starts_with(p))
        }
        "python" => {
            let prefixes: &[&str] = &["def ", "async def ", "class "];
            prefixes.iter().any(|p| line.starts_with(p))
        }
        "ts" => {
            let prefixes: &[&str] = &[
                "export function ",
                "export default function ",
                "export async function ",
                "function ",
                "async function ",
                "export class ",
                "export default class ",
                "class ",
                "export const ",
                "export let ",
                "export var ",
                "export type ",
                "export interface ",
                "export enum ",
                "export default ",
                "const ",
                "interface ",
                "type ",
            ];
            prefixes.iter().any(|p| line.starts_with(p))
        }
        "go" => {
            let prefixes: &[&str] = &["func ", "type ", "package "];
            prefixes.iter().any(|p| line.starts_with(p))
        }
        "java" | "kotlin" => {
            let prefixes: &[&str] = &[
                "public class ",
                "private class ",
                "protected class ",
                "public interface ",
                "private interface ",
                "protected interface ",
                "class ",
                "interface ",
                "public static ",
                "fun ",
                "public fun ",
                "private fun ",
                "protected fun ",
                "public enum ",
                "private enum ",
                "enum ",
            ];
            prefixes.iter().any(|p| line.starts_with(p))
        }
        "ruby" | "swift" => {
            let prefixes: &[&str] = &["def ", "class ", "module ", "func ", "struct ", "enum "];
            prefixes.iter().any(|p| line.starts_with(p))
        }
        "c" => {
            // Heuristic: non-whitespace-leading line containing '(' ending with ')' or ') {'
            if line.starts_with(|c: char| c.is_whitespace()) {
                return false;
            }
            let trimmed = line.trim_end();
            (trimmed.ends_with(')') || trimmed.ends_with(") {") || trimmed.ends_with("){"))
                && line.contains('(')
                && line.chars().next().map_or(false, |c| c.is_ascii_alphabetic() || c == '_')
        }
        _ => false,
    }
}

/// Compress to signature lines, preserving first-3 and last-3 verbatim,
/// plus indented method signatures inside `impl`/`class` blocks and the
/// doc-comment / attribute / decorator run that immediately precedes each
/// kept signature.
fn compress_signatures(lines: Vec<String>, lang: &str) -> Vec<String> {
    let total = lines.len();
    let anchor_count = 3.min(total / 2); // don't overlap on tiny files

    let first_lines: Vec<String> = lines[..anchor_count].to_vec();
    let last_lines: Vec<String> = lines[total.saturating_sub(anchor_count)..].to_vec();

    let head_end = anchor_count;
    let tail_start = total.saturating_sub(anchor_count);

    let mut middle: Vec<String> = Vec::new();
    let mut pending: Vec<String> = Vec::new();
    for (i, line) in lines.iter().enumerate() {
        if i < head_end || i >= tail_start {
            // Anchors carry their own context — no need to buffer doc lines.
            pending.clear();
            continue;
        }
        let s = line.as_str();
        if is_doc_or_attribute(s, lang) {
            pending.push(line.clone());
            continue;
        }
        if is_signature(s, lang) || is_indented_signature(s, lang) {
            middle.extend(pending.drain(..));
            middle.push(line.clone());
            continue;
        }
        // Any non-doc, non-signature line breaks the accumulating context.
        pending.clear();
    }

    let mut out = Vec::with_capacity(anchor_count * 2 + middle.len());
    out.extend(first_lines);
    out.extend(middle);
    out.extend(last_lines);
    out
}

impl Handler for FsHandler {
    fn compress(&self, cmd: &str, lines: Vec<String>, config: &Config) -> Vec<String> {
        // ── Signature-mode ────────────────────────────────────────────────
        // Check on raw lines so the reported K reflects actual file size,
        // not post-filter count.
        if config.sig_mode_enabled {
            if let Some((file_path, lang)) = extract_path_and_ext(cmd) {
                let k = lines.len();
                if k >= config.sig_mode_threshold_lines {
                    let filtered = smart_filter::apply(lines);
                    let compressed = compress_signatures(filtered, lang);
                    let n_sigs = compressed.len();
                    let marker = format!(
                        "[squeez: sig-mode {} signatures from {} lines — use `sed -n A,Bp {}` for a range]",
                        n_sigs, k, file_path
                    );
                    let mut out = Vec::with_capacity(n_sigs + 1);
                    out.push(marker);
                    out.extend(compressed);
                    return out;
                }
            }
        }

        let lines = smart_filter::apply(lines);

        // ── Existing env/find/ls path ─────────────────────────────────────
        if cmd.trim_start().starts_with("env") || cmd.contains("printenv") {
            let filtered: Vec<String> = lines
                .into_iter()
                .filter(|l| !NOISY_ENV_PREFIXES.iter().any(|p| l.starts_with(p)))
                .collect();
            return truncation::apply(filtered, 80, truncation::Keep::Head);
        }

        let lines = grouping::group_files_by_dir(lines, 5);
        truncation::apply(lines, config.find_max_results, truncation::Keep::Head)
    }
}
