#![cfg(feature="reload")]
#![hot_lib_reloader::hot_module(dylib = "../iced_ui/theme")]

pub use iced_ui::run;

hot_functions_from_file!("crates/iced_ui/theme/styles.rs");
