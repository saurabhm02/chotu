mod api;
mod app;
mod components;
mod hooks;
mod models;
mod utils;

use app::App;

fn main() {
    console_error_panic_hook::set_once();
    wasm_logger::init(wasm_logger::Config::default());
    yew::Renderer::<App>::new().render();
}
