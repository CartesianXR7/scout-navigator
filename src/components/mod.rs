// src/components/mod.rs

pub mod canvas;
pub mod controls;
pub mod help_bubble;
pub mod main_app;

// Re-export MainApp so it can be used as components::MainApp
pub use self::main_app::MainApp;
