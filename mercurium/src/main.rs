#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

#[cfg(not(feature = "slint"))]
fn main() {
    #[cfg(debug_assertions)]
    std::env::set_var("RUST_BACKTRACE", "1");

    #[cfg(any(target_os = "windows", target_os = "linux", target_os = "macos"))]
    iced_ui::App::run().unwrap();
}

#[cfg(feature = "slint")]
fn main() -> Result<(), slint::PlatformError> {

}