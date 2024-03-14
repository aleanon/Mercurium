#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod hot_lib;

fn main() {
    
    #[cfg(not(feature = "relaod"))]
    iced_ui::run().unwrap();

    #[cfg(feature = "reload")]
    hot_lib::run()

}
