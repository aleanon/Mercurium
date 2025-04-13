
use deps::*;

#[cfg(not(feature = "reload"))]
pub use dynrule::*;

#[cfg(feature = "reload")]
pub use hot_module::*;

#[cfg(feature = "reload")]
#[hot_lib_reloader::hot_module(dylib = "dynrule", lib_dir = "target/reload")]
mod hot_module {
    use dynrule::*;
    hot_functions_from_file!("iced_ui/styles/rule/src/lib.rs", ignore_no_mangle = true);
}