#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]


#[cfg(feature = "reload")]
#[hot_lib_reloader::hot_module(dylib = "styles")]
mod hot_lib {
    use styles::*;
    // println!("hot reloading");
    hot_functions_from_file!("iced_ui/styles/src/lib.rs", ignore_no_mangle = true);
    
    // hot_functions_from_file!("iced_ui/theme/src/styles/container.rs", ignore_no_mangle = true);
}

// #[cfg(feature="reload")]
// use hot_lib::*;

#[tokio::main]
async fn main() -> Result<(), iced_ui::Error> {
    #[cfg(all(debug_assertions, target_os = "windows"))]
    unsafe{std::env::set_var("RUST_BACKTRACE", "1")};


    #[cfg(any(target_os = "windows", target_os = "linux", target_os = "macos"))]
    iced_ui::App::run()
}

// fn main() {
//     loop {
//         println!("1");

//         thread::sleep(Duration::from_millis(10000));
//     }
// }

