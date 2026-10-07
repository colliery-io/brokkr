//! Every view shows a failed panel with the local `PanelError`, not with
//! Aurora's `ErrorState` directly. `PanelError` is the one place that shows a
//! refused token as a neutral wait under the session banner, so a view that
//! uses `ErrorState` would show the raw "Not authorized" again.

use std::path::Path;

#[test]
fn no_view_uses_error_state_directly() {
    let dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("src/views");
    let mut bad = Vec::new();
    for entry in std::fs::read_dir(&dir).expect("read views") {
        let p = entry.expect("dir entry").path();
        if p.extension().and_then(|e| e.to_str()) != Some("rs") {
            continue;
        }
        let text = std::fs::read_to_string(&p).expect("read file");
        for (n, line) in text.lines().enumerate() {
            if line.contains("<ErrorState") {
                bad.push(format!("{}:{}", p.display(), n + 1));
            }
        }
    }
    assert!(
        bad.is_empty(),
        "views that use ErrorState, not PanelError:\n{}",
        bad.join("\n")
    );
}
