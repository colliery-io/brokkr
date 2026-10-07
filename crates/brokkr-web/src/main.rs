//! Brokkr Operator Console — Leptos CSR entrypoint (BROKKR-I-0031).

mod api;
mod app;
mod components;
mod live;
mod models;
mod views;

fn main() {
    // Log a panic's message to the console; without it a WASM panic shows only
    // as "unreachable".
    std::panic::set_hook(Box::new(|info| leptos::logging::error!("panic: {info}")));
    leptos::mount::mount_to_body(app::App);
}
