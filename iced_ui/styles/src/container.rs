// #[cfg(not(feature = "reload"))]
pub use dyncontainer::*;

// #[cfg(feature = "reload")]
// pub use hot_module::*;

// #[cfg(feature = "reload")]
// #[hot_lib_reloader::hot_module(dylib = "dyncontainer", lib_dir = "target/reload")]
// mod hot_module {
//     use dyncontainer::*;
//     hot_functions_from_file!("iced_ui/styles/container/src/lib.rs", ignore_no_mangle = true);
// }
