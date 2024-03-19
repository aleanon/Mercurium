#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod hot_lib;

fn main() {
    
    #[cfg(all(not(feature = "reload"), any(target_os = "windows", target_os = "linux", target_os = "macos")))]
    iced_ui::run().unwrap();

    #[cfg(feature = "reload")]
    hot_lib::run()

}
