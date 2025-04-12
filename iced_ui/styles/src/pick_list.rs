
use deps::hot_lib_reloader;

#[cfg(not(feature = "reload"))]
pub use dynpick_list::*;

#[cfg(feature = "reload")]
pub use hot_module::*;

#[cfg(feature = "reload")]
#[hot_lib_reloader::hot_module(dylib = "dynpick_list", lib_dir = "target/reload")]
mod hot_module {
    use dynpick_list::*;
    hot_functions_from_file!("iced_ui/styles/pick_list/src/lib.rs", ignore_no_mangle = true);
}