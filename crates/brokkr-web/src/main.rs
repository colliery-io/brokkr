//! Brokkr Operator Console — Leptos CSR entrypoint (BROKKR-I-0031).

mod api;
mod app;
mod components;
mod models;
mod views;

fn main() {
    leptos::mount::mount_to_body(app::App);
}
