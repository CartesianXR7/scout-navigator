// src/lib.rs

#![allow(deprecated)]

use wasm_bindgen::prelude::*;
use yew::Renderer;

mod components;
mod pathfinding;
mod rover;

use components::MainApp;

#[wasm_bindgen]
pub fn greet(name: &str) -> String {
    format!("Hello, {}! Scout Pathfinder is working.", name)
}

#[wasm_bindgen]
pub fn run_app() {
    console_error_panic_hook::set_once();

    web_sys::console::log_1(&"Scout Pathfinder starting initialization...".into());

    let window = web_sys::window().expect("no global `window` exists");
    let document = window.document().expect("no global `document` exists");

    web_sys::console::log_1(&"Window and document obtained".into());

    let app_element = document
        .get_element_by_id("app")
        .expect("could not find element with id 'app'");

    web_sys::console::log_1(&"App element found".into());

    web_sys::console::log_1(&"🔧 Attempting to mount Yew application...".into());

    Renderer::<MainApp>::with_root(app_element).render();

    web_sys::console::log_1(&"Scout Pathfinder initialized successfully!".into());
}
