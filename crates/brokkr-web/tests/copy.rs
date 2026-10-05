//! No user-facing string names a work item. A Kairos or Metis code such as
//! `BROKKR-T-0300` belongs in a comment or a commit, not on screen
//! (BROKKR-T-0338).

use std::path::{Path, PathBuf};

fn rust_files(dir: &Path, out: &mut Vec<PathBuf>) {
    for entry in std::fs::read_dir(dir).expect("read dir") {
        let p = entry.expect("dir entry").path();
        if p.is_dir() {
            rust_files(&p, out);
        } else if p.extension().and_then(|e| e.to_str()) == Some("rs") {
            out.push(p);
        }
    }
}

/// `PREFIX-L-NNN`: two or more capitals, one capital, three or more digits.
fn is_work_item_code(token: &str) -> bool {
    let mut parts = token.split('-');
    let (Some(prefix), Some(letter), Some(number), None) =
        (parts.next(), parts.next(), parts.next(), parts.next())
    else {
        return false;
    };
    prefix.len() >= 2
        && prefix.chars().all(|c| c.is_ascii_uppercase())
        && letter.len() == 1
        && letter.chars().all(|c| c.is_ascii_uppercase())
        && number.len() >= 3
        && number.chars().all(|c| c.is_ascii_digit())
}

/// A work-item code on the line, outside a `//` comment. Code outside a
/// comment is Rust, and Rust names a work item only inside a string.
fn code_outside_comments(line: &str) -> Option<String> {
    let code = line.split("//").next().unwrap_or("");
    code.split(|c: char| !(c.is_ascii_alphanumeric() || c == '-'))
        .find(|t| is_work_item_code(t))
        .map(str::to_string)
}

#[test]
fn no_work_item_code_in_a_user_facing_string() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let mut files = Vec::new();
    rust_files(&root.join("src"), &mut files);
    let mut bad = Vec::new();
    for f in &files {
        let text = std::fs::read_to_string(f).expect("read file");
        for (n, line) in text.lines().enumerate() {
            if let Some(code) = code_outside_comments(line) {
                bad.push(format!("{}:{}: {code}", f.display(), n + 1));
            }
        }
    }
    assert!(
        bad.is_empty(),
        "work-item codes on screen:\n{}",
        bad.join("\n")
    );
}

#[test]
fn the_scanner_reads_codes_and_skips_comments() {
    assert_eq!(
        code_outside_comments(r#""see (BROKKR-T-0300), so""#).as_deref(),
        Some("BROKKR-T-0300")
    );
    assert_eq!(code_outside_comments("    // fixed in BROKKR-T-0300"), None);
    assert_eq!(
        code_outside_comments(r#"let x = 1; // BROKKR-T-0300"#),
        None
    );
    assert_eq!(
        code_outside_comments(r#""https://colliery-io.github.io/brokkr""#),
        None
    );
    assert_eq!(code_outside_comments(r#""UTF-8 and ISO-8601""#), None);
    assert!(is_work_item_code("AURORA-T-0007"));
    assert!(!is_work_item_code("ws-url-x"));
    assert!(!is_work_item_code("A-T-0007"));
}
