//! Brokkr Operator Console — Leptos CSR entrypoint (BROKKR-I-0031, slice 1a).

mod app;

fn main() {
    leptos::mount::mount_to_body(app::App);
}
