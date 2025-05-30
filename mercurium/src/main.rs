#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

#[cfg(any(target_os = "windows", target_os = "linux", target_os = "macos"))]
mod run_with_iced;

fn main() -> Result<(), iced_ui::Error> {
    #[cfg(all(debug_assertions, target_os = "windows"))]
    unsafe {
        std::env::set_var("RUST_BACKTRACE", "1")
    };

    // #[cfg(debug_assertions)]
    // deps::simple_logger::SimpleLogger::new().env().init().unwrap();

    #[cfg(any(target_os = "windows", target_os = "linux", target_os = "macos"))]
    run_with_iced::run()
}
