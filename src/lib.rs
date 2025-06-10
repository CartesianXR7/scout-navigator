// src/lib.rs

use wasm_bindgen::prelude::*;
use yew::Renderer;

mod components;
mod pathfinding;
mod rover;

// Use the original MainApp component
use components::MainApp;

// Add this to expose a simple test function
#[wasm_bindgen]
pub fn greet(name: &str) -> String {
    format!("Hello, {}! Scout Pathfinder is working.", name)
}

// Remove the (start) attribute and make it a regular function
#[wasm_bindgen]
pub fn run_app() {
    // Set panic hook for better error messages
    console_error_panic_hook::set_once();
    
    // Log initialization start
    web_sys::console::log_1(&"🚀 Scout Pathfinder starting initialization...".into());
    
    // Get window and document
    let window = web_sys::window().expect("no global `window` exists");
    let document = window.document().expect("no global `document` exists");
    
    web_sys::console::log_1(&"✅ Window and document obtained".into());
    
    // Find the app container
    let app_element = document
        .get_element_by_id("app")
        .expect("could not find element with id 'app'");
    
    web_sys::console::log_1(&"✅ App element found".into());
    
    // Mount the Yew application
    web_sys::console::log_1(&"🔧 Attempting to mount Yew application...".into());
    
    Renderer::<MainApp>::with_root(app_element).render();
    
    web_sys::console::log_1(&"🚀 Scout Pathfinder initialized successfully!".into());
}