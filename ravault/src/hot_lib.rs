#![cfg(feature="reload")]
#![cfg(debug_assertions)]
#[hot_lib_reloader::hot_module(dylib = "../iced_ui")]

pub use iced_ui::run;

hot_functions_from_file!("crates/iced_ui/src/view.rs");
