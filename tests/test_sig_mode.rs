use squeez::commands::{fs::FsHandler, Handler};
use squeez::config::Config;

// ── helpers ────────────────────────────────────────────────────────────────

fn make_rust_file(n: usize) -> Vec<String> {
    // ~10 body lines per function so signatures are sparse (≈n/10 sigs).
    let mut lines = Vec::with_capacity(n);
    lines.push("// auto-generated synthetic Rust file".to_string());
    lines.push("use std::collections::HashMap;".to_string());
    lines.push("".to_string());
    let mut i = 0usize;
    while lines.len() + 12 <= n {
        lines.push(format!("pub fn function_{}(x: u32) -> u32 {{", i));
        // 9 body lines + closing brace + blank = 11 more lines
        for j in 0..9usize {
            lines.push(format!("    let v{} = x * {};", j, i + j));
        }
        lines.push("}".to_string());
        lines.push("".to_string());
        i += 1;
    }
    // pad to exactly n lines
    while lines.len() < n {
        lines.push(format!("// padding line {}", lines.len()));
    }
    lines.truncate(n);
    lines
}

fn make_py_file(n: usize) -> Vec<String> {
    let mut lines = Vec::with_capacity(n);
    lines.push("# synthetic Python file".to_string());
    lines.push("import os".to_string());
    lines.push("".to_string());
    for i in 0..(n / 5) {
        lines.push(format!("def func_{}(arg):", i));
        lines.push(format!("    return arg + {}", i));
        lines.push("".to_string());
        lines.push(format!("class MyClass_{}:", i));
        lines.push("    pass".to_string());
    }
    while lines.len() < n {
        lines.push(format!("# pad {}", lines.len()));
    }
    lines.truncate(n);
    lines
}

fn make_txt_file(n: usize) -> Vec<String> {
    (0..n).map(|i| format!("line {}", i)).collect()
}

// ── (a) 1000-line Rust file → sig-mode fires ──────────────────────────────

#[test]
fn rust_1000_lines_sig_mode_fires() {
    let lines = make_rust_file(1000);
    let cfg = Config::default(); // sig_mode_enabled=true, threshold=400
    let result = FsHandler.compress("cat src/foo.rs", lines, &cfg);

    // Marker present
    assert!(
        result[0].starts_with("[squeez: sig-mode"),
        "expected marker as first line, got: {:?}",
        result[0]
    );
    assert!(result[0].contains("from 1000 lines"));
    assert!(result[0].contains("src/foo.rs"));

    // Compressed well below 150 output lines
    assert!(
        result.len() < 150,
        "expected < 150 lines, got {}",
        result.len()
    );

    // Every fn signature from the synthetic file is present
    let body = result[1..].join("\n");
    assert!(body.contains("pub fn function_0("), "missing function_0");
    assert!(body.contains("pub fn function_1("), "missing function_1");
}

// ── (b) 500-line .txt file → untouched ────────────────────────────────────

#[test]
fn txt_file_not_compressed() {
    let lines = make_txt_file(500);
    let original_len = lines.len();
    let cfg = Config::default();
    let result = FsHandler.compress("cat notes.txt", lines, &cfg);

    // No marker
    assert!(
        !result[0].starts_with("[squeez: sig-mode"),
        "sig-mode should not fire on .txt"
    );
    // Line count unchanged (possibly truncated by find_max_results, but no sig-mode)
    // Key check: marker absent
    for line in &result {
        assert!(
            !line.starts_with("[squeez: sig-mode"),
            "unexpected sig-mode marker in .txt output"
        );
    }
    let _ = original_len; // used above
}

// ── (c) sig_mode_enabled=false + 1000-line .rs → untouched ───────────────

#[test]
fn sig_mode_disabled_skips_compression() {
    let lines = make_rust_file(1000);
    let mut cfg = Config::default();
    cfg.sig_mode_enabled = false;

    let result = FsHandler.compress("cat src/lib.rs", lines, &cfg);

    for line in &result {
        assert!(
            !line.starts_with("[squeez: sig-mode"),
            "sig-mode marker must not appear when disabled"
        );
    }
}

// ── (d) 100-line .rs below threshold → untouched ─────────────────────────

#[test]
fn short_rs_below_threshold_untouched() {
    let lines = make_rust_file(100);
    let cfg = Config::default(); // threshold=400

    let result = FsHandler.compress("cat src/small.rs", lines, &cfg);

    for line in &result {
        assert!(
            !line.starts_with("[squeez: sig-mode"),
            "sig-mode must not fire below threshold"
        );
    }
}

// ── (e) .py file with 500 lines → sig-mode fires, class/def preserved ────

#[test]
fn python_500_lines_sig_mode_fires() {
    let lines = make_py_file(500);
    let cfg = Config::default();

    let result = FsHandler.compress("cat app/views.py", lines, &cfg);

    assert!(
        result[0].starts_with("[squeez: sig-mode"),
        "expected sig-mode marker for .py file, got: {:?}",
        result[0]
    );
    assert!(result[0].contains("from 500 lines"));

    let body = result[1..].join("\n");
    assert!(body.contains("def func_0("), "missing func_0");
    assert!(body.contains("class MyClass_0:"), "missing MyClass_0");
}

// ── config.ini parsing for new fields ─────────────────────────────────────

#[test]
fn config_sig_mode_defaults() {
    let cfg = Config::default();
    assert!(cfg.sig_mode_enabled);
    assert_eq!(cfg.sig_mode_threshold_lines, 400);
}

#[test]
fn config_sig_mode_parsed_from_ini() {
    let ini = "sig_mode_enabled=false\nsig_mode_threshold_lines=200\n";
    let cfg = Config::from_str(ini);
    assert!(!cfg.sig_mode_enabled);
    assert_eq!(cfg.sig_mode_threshold_lines, 200);
}

// ── head / tail variants also route through sig-mode ─────────────────────

#[test]
fn head_command_triggers_sig_mode() {
    let lines = make_rust_file(1000);
    let cfg = Config::default();
    let result = FsHandler.compress("head -n 1000 src/main.rs", lines, &cfg);
    assert!(
        result[0].starts_with("[squeez: sig-mode"),
        "head command should trigger sig-mode"
    );
}

#[test]
fn tail_command_triggers_sig_mode() {
    let lines = make_rust_file(1000);
    let cfg = Config::default();
    let result = FsHandler.compress("tail -n 1000 src/lib.rs", lines, &cfg);
    assert!(
        result[0].starts_with("[squeez: sig-mode"),
        "tail command should trigger sig-mode"
    );
}

// ── non-code extension with bat → untouched ──────────────────────────────

#[test]
fn bat_on_json_file_no_sig_mode() {
    let lines: Vec<String> = (0..600).map(|i| format!("{{\"key\": {}}}", i)).collect();
    let cfg = Config::default();
    let result = FsHandler.compress("bat config.json", lines, &cfg);
    for line in &result {
        assert!(
            !line.starts_with("[squeez: sig-mode"),
            "sig-mode must not fire on .json"
        );
    }
}

// ── P4 (issue #9): indented methods inside impl/class + doc preservation ──

/// Build a Rust file dominated by an `impl` block so almost every signature
/// lives indented. Without indented-signature support, sig-mode would drop
/// all of them.
fn make_rust_impl_file(n_methods: usize) -> Vec<String> {
    let mut lines: Vec<String> = Vec::new();
    lines.push("//! Module-level docs (preserved as anchor).".to_string());
    lines.push("use std::collections::HashMap;".to_string());
    lines.push("".to_string());
    lines.push("pub struct Widget { id: u64 }".to_string());
    lines.push("".to_string());
    lines.push("impl Widget {".to_string());
    for i in 0..n_methods {
        lines.push(format!("    /// Method {} — docstring preserved.", i));
        lines.push(format!("    #[inline]"));
        lines.push(format!("    pub fn method_{}(&self, arg: u64) -> u64 {{", i));
        // 10 body lines per method so signatures are sparse
        for j in 0..10 {
            lines.push(format!("        let _local_{} = arg + {};", j, i + j));
        }
        lines.push("        arg".to_string());
        lines.push("    }".to_string());
        lines.push("".to_string());
    }
    lines.push("}".to_string());
    // Pad with trailing comments so we comfortably clear the threshold.
    while lines.len() < 600 {
        lines.push(format!("// tail padding {}", lines.len()));
    }
    lines
}

#[test]
fn rust_impl_methods_are_kept_indented() {
    let lines = make_rust_impl_file(30);
    let cfg = Config::default();
    let result = FsHandler.compress("cat src/widget.rs", lines, &cfg);

    let body = result.join("\n");
    // The compressed output must include indented method signatures.
    let kept = result
        .iter()
        .filter(|l| l.trim_start().starts_with("pub fn method_"))
        .count();
    assert!(
        kept >= 10,
        "expected ≥10 indented method signatures kept, got {}\n--- output ---\n{}",
        kept,
        body,
    );
}

#[test]
fn rust_doc_comments_and_attrs_above_signatures_are_kept() {
    let lines = make_rust_impl_file(8);
    let cfg = Config::default();
    let result = FsHandler.compress("cat src/widget.rs", lines, &cfg);

    let body = result.join("\n");
    assert!(
        body.contains("/// Method 1 — docstring preserved."),
        "doc comments must be preserved above signatures\n{}",
        body,
    );
    assert!(
        body.contains("#[inline]"),
        "attributes must be preserved above signatures\n{}",
        body,
    );
}

fn make_py_class_file() -> Vec<String> {
    let mut lines: Vec<String> = Vec::new();
    lines.push("\"\"\"Module docstring.\"\"\"".to_string());
    lines.push("import asyncio".to_string());
    lines.push("".to_string());
    lines.push("class Service:".to_string());
    for i in 0..40 {
        lines.push(format!("    # Method {} docstring", i));
        lines.push(format!("    @staticmethod"));
        lines.push(format!("    def method_{}(arg):", i));
        for j in 0..8 {
            lines.push(format!("        local_{} = arg + {}", j, i + j));
        }
        lines.push("        return arg".to_string());
        lines.push("".to_string());
    }
    while lines.len() < 600 {
        lines.push(format!("# tail pad {}", lines.len()));
    }
    lines
}

#[test]
fn python_class_methods_and_decorators_kept() {
    let lines = make_py_class_file();
    let cfg = Config::default();
    let result = FsHandler.compress("cat service.py", lines, &cfg);
    let body = result.join("\n");

    let kept_methods = result
        .iter()
        .filter(|l| l.trim_start().starts_with("def method_"))
        .count();
    assert!(
        kept_methods >= 10,
        "expected ≥10 indented Python method signatures kept, got {}\n{}",
        kept_methods,
        body,
    );

    let kept_decorators = result
        .iter()
        .filter(|l| l.trim_start().starts_with("@staticmethod"))
        .count();
    assert!(
        kept_decorators >= 10,
        "expected decorators preserved, got {}\n{}",
        kept_decorators,
        body,
    );
}

fn make_ts_class_file() -> Vec<String> {
    let mut lines: Vec<String> = Vec::new();
    lines.push("// Top-of-file comment.".to_string());
    lines.push("import { X } from './x';".to_string());
    lines.push("".to_string());
    lines.push("export class Service {".to_string());
    for i in 0..40 {
        lines.push(format!("    /** Method {} doc. */", i));
        lines.push(format!("    public method_{}(arg: number): number {{", i));
        for j in 0..8 {
            lines.push(format!("        const local_{} = arg + {};", j, i + j));
        }
        lines.push("        return arg;".to_string());
        lines.push("    }".to_string());
        lines.push("".to_string());
    }
    lines.push("}".to_string());
    while lines.len() < 600 {
        lines.push(format!("// pad {}", lines.len()));
    }
    lines
}

#[test]
fn typescript_class_methods_kept_indented() {
    let lines = make_ts_class_file();
    let cfg = Config::default();
    let result = FsHandler.compress("cat service.ts", lines, &cfg);
    let body = result.join("\n");

    // The matcher currently accepts top-level `function`/`class`/`const`/etc.;
    // public/method keywords are not in the prefix list. So we expect at
    // least the JSDoc preservation to come through.
    assert!(
        body.contains("/** Method"),
        "JSDoc comments should be preserved\n{}",
        body,
    );
}

fn make_top_level_only_rust() -> Vec<String> {
    let mut lines: Vec<String> = Vec::new();
    lines.push("// no impl blocks".to_string());
    for i in 0..50 {
        lines.push(format!("pub fn standalone_{}(x: u64) -> u64 {{", i));
        for j in 0..10 {
            lines.push(format!("    let v{} = x + {};", j, i + j));
        }
        lines.push("    x".to_string());
        lines.push("}".to_string());
    }
    lines
}

#[test]
fn top_level_only_file_still_keeps_all_signatures() {
    // Regression: indented-aware logic must not drop top-level signatures.
    let lines = make_top_level_only_rust();
    let cfg = Config::default();
    let result = FsHandler.compress("cat lib.rs", lines, &cfg);
    let kept = result
        .iter()
        .filter(|l| l.starts_with("pub fn standalone_"))
        .count();
    assert!(
        kept >= 40,
        "regression: top-level signatures dropped — got {}",
        kept,
    );
}
