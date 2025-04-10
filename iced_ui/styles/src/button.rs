#[cfg(not(feature = "reload"))]
pub use dynbutton::*;

#[cfg(feature = "reload")]
pub use hot_module::*;

#[cfg(feature = "reload")]
#[hot_lib_reloader::hot_module(dylib = "dynbutton", lib_dir = "target/reload")]
mod hot_module {
    use dynbutton::*;
    hot_functions_from_file!("iced_ui/styles/button/src/lib.rs", ignore_no_mangle = true);
}