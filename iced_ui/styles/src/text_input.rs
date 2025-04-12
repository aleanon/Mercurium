
use deps::hot_lib_reloader;

#[cfg(not(feature = "reload"))]
pub use dyntext_input::*;

#[cfg(feature = "reload")]
pub use hot_module::*;

#[cfg(feature = "reload")]
#[hot_lib_reloader::hot_module(dylib = "dyntext_input", lib_dir = "target/reload")]
mod hot_module {
    use dyntext_input::*;
    hot_functions_from_file!("iced_ui/styles/text_input/src/lib.rs", ignore_no_mangle = true);
}
