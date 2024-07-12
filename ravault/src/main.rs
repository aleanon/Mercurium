#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::env;

mod hot_lib;

fn main() {
    #[cfg(debug_assertions)]
    env::set_var("RUST_BACKTRACE", "1");

    #[cfg(all(
        not(feature = "reload"),
        any(target_os = "windows", target_os = "linux", target_os = "macos")
    ))]
    iced_ui::run().unwrap();

    #[cfg(feature = "reload")]
    hot_lib::run()
}
