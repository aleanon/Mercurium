#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]


#[tokio::main]
async fn main() -> Result<(), iced_ui::Error> {
    #[cfg(all(debug_assertions, target_os = "windows"))]
    unsafe{std::env::set_var("RUST_BACKTRACE", "1")};


    #[cfg(any(target_os = "windows", target_os = "linux", target_os = "macos"))]
    iced_ui::App::run()
}
