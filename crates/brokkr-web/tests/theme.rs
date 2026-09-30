//! Both Aurora themes must work: the console may hold no colour of its
//! own (Aurora's tokens.css is the only place with raw colours), and the
//! page must set its theme before the first paint.

use std::path::{Path, PathBuf};

fn files(dir: &Path, out: &mut Vec<PathBuf>) {
    for entry in std::fs::read_dir(dir).expect("read dir") {
        let p = entry.expect("dir entry").path();
        if p.is_dir() {
            files(&p, out);
        } else if matches!(
            p.extension().and_then(|e| e.to_str()),
            Some("rs" | "css" | "html")
        ) {
            out.push(p);
        }
    }
}

/// `#abc`, `#abcd`, `#aabbcc` or `#aabbccdd` followed by a non-word char.
fn hex_colours(text: &str) -> Vec<String> {
    let b = text.as_bytes();
    let mut hits = Vec::new();
    for (i, _) in text.match_indices('#') {
        let n = b[i + 1..]
            .iter()
            .take_while(|c| c.is_ascii_hexdigit())
            .count();
        let next = b.get(i + 1 + n).copied().unwrap_or(b' ');
        if matches!(n, 3 | 4 | 6 | 8) && !(next.is_ascii_alphanumeric() || next == b'_') {
            hits.push(text[i..i + 1 + n].to_string());
        }
    }
    hits
}

#[test]
fn no_raw_colours_in_the_console() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let mut all = Vec::new();
    for d in ["src", "style"] {
        files(&root.join(d), &mut all);
    }
    all.push(root.join("index.html"));
    let fns = ["rgb(", "rgba(", "hsl("];
    let mut bad = Vec::new();
    for f in &all {
        let text = std::fs::read_to_string(f).expect("read file");
        for (n, line) in text.lines().enumerate() {
            let mut found = hex_colours(line);
            found.extend(
                fns.iter()
                    .filter(|f| line.contains(**f))
                    .map(|f| f.to_string()),
            );
            // A token with a raw fallback, `var(--x, <colour>)`.
            if line.contains("var(--") && line.contains(", #") {
                found.push("var() fallback".into());
            }
            for h in found {
                bad.push(format!("{}:{}: {h}", f.display(), n + 1));
            }
        }
    }
    assert!(bad.is_empty(), "raw colours:\n{}", bad.join("\n"));
}

#[test]
fn hex_scanner_finds_colours_and_skips_fragments() {
    assert_eq!(hex_colours("stroke=\"#0b0d10\""), vec!["#0b0d10"]);
    assert_eq!(hex_colours("a #fff; b"), vec!["#fff"]);
    assert!(hex_colours("href=\"#fleet\" #{id}").is_empty());
}

#[test]
fn index_html_sets_the_theme_before_first_paint() {
    let html = include_str!("../index.html");
    assert!(
        html.contains(aurora_leptos::THEME_INIT_SCRIPT),
        "index.html must inline aurora_leptos::THEME_INIT_SCRIPT"
    );
    assert!(html.contains(r#"name="color-scheme" content="light dark""#));
}
