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

/// A char that can continue an identifier: `white-space`, `fg_bright`, `rgb2`.
fn is_word(c: u8) -> bool {
    c.is_ascii_alphanumeric() || c == b'_' || c == b'-'
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

/// A CSS colour function, `rgb(` to `oklch(`, as a whole word: `color-mix(`
/// and `var(--lab(` are not colours.
fn colour_functions(text: &str) -> Vec<String> {
    const FNS: [&str; 10] = [
        "rgb(", "rgba(", "hsl(", "hsla(", "hwb(", "lab(", "lch(", "oklab(", "oklch(", "color(",
    ];
    let b = text.as_bytes();
    let mut hits = Vec::new();
    for f in FNS {
        for (i, _) in text.match_indices(f) {
            let prev = if i == 0 { b' ' } else { b[i - 1] };
            if !is_word(prev) {
                hits.push(f.to_string());
            }
        }
    }
    hits
}

/// A common named colour where CSS reads a value: after `:`, `,`, `(`, `=`
/// or a quote, and not inside a longer word (`white-space`, `--gold-fg`).
fn named_colours(text: &str) -> Vec<String> {
    const NAMES: [&str; 22] = [
        "red", "white", "black", "blue", "green", "yellow", "orange", "purple", "pink", "brown",
        "gray", "grey", "cyan", "magenta", "silver", "navy", "maroon", "olive", "lime", "aqua",
        "fuchsia", "indigo",
    ];
    let b = text.as_bytes();
    let mut hits = Vec::new();
    for name in NAMES {
        for (i, _) in text.match_indices(name) {
            let end = i + name.len();
            let prev = if i == 0 { b' ' } else { b[i - 1] };
            let next = b.get(end).copied().unwrap_or(b' ');
            if is_word(prev) || is_word(next) {
                continue;
            }
            let before = text[..i].trim_end_matches(' ');
            let in_value = matches!(
                before.as_bytes().last(),
                Some(b':' | b',' | b'(' | b'=' | b'"' | b'\'')
            );
            if in_value {
                hits.push(name.to_string());
            }
        }
    }
    hits
}

fn raw_colours(line: &str) -> Vec<String> {
    let mut found = hex_colours(line);
    found.extend(colour_functions(line));
    found.extend(named_colours(line));
    // A token with a raw fallback, `var(--x, <colour>)`.
    if line.contains("var(--") && line.contains(", #") {
        found.push("var() fallback".into());
    }
    found
}

#[test]
fn no_raw_colours_in_the_console() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let mut all = Vec::new();
    for d in ["src", "style"] {
        files(&root.join(d), &mut all);
    }
    all.push(root.join("index.html"));
    let mut bad = Vec::new();
    for f in &all {
        let text = std::fs::read_to_string(f).expect("read file");
        for (n, line) in text.lines().enumerate() {
            for h in raw_colours(line) {
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
fn function_scanner_finds_every_colour_space() {
    for f in [
        "rgb(", "rgba(", "hsl(", "hsla(", "hwb(", "lab(", "lch(", "oklab(", "oklch(", "color(",
    ] {
        let line = format!("color: {f}1 2 3);");
        assert_eq!(colour_functions(&line), vec![f.to_string()], "{line}");
    }
    assert!(
        colour_functions("background: color-mix(in srgb, var(--ice) 12%, transparent);").is_empty()
    );
    assert!(colour_functions("let slab(x) = 1;").is_empty());
}

#[test]
fn name_scanner_finds_values_and_skips_words() {
    assert_eq!(named_colours("color: red;"), vec!["red"]);
    assert_eq!(
        named_colours("border: 1px solid white"),
        Vec::<String>::new()
    );
    assert_eq!(named_colours("fill=\"black\""), vec!["black"]);
    assert_eq!(
        named_colours("background: linear-gradient(blue, green)"),
        vec!["blue", "green"]
    );
    assert!(named_colours("white-space: nowrap;").is_empty());
    assert!(named_colours("color: var(--gold-fg);").is_empty());
    assert!(named_colours("// the red dot pulses while a beat is fresh").is_empty());
    assert!(named_colours("let greyhound = 1;").is_empty());
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
