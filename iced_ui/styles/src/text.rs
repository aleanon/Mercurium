
#[cfg(not(feature = "reload"))]
pub use dyntext::*;

#[cfg(feature = "reload")]
pub use hot_module::*;

#[cfg(feature = "reload")]
#[hot_lib_reloader::hot_module(dylib = "dyntext", lib_dir = "target/reload")]
mod hot_module {
    use dyntext::*;
    hot_functions_from_file!("iced_ui/styles/text/src/lib.rs", ignore_no_mangle = true);
}
