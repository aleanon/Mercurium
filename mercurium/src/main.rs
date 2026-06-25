#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use tracing_error::ErrorLayer;
use tracing_subscriber::{EnvFilter, fmt, layer::SubscriberExt, util::SubscriberInitExt};

#[cfg(any(target_os = "windows", target_os = "linux", target_os = "macos"))]
mod run_with_iced;

fn main() -> Result<(), iced_ui::Error> {
    init_tracing();
    #[cfg(all(debug_assertions, target_os = "windows"))]
    unsafe {
        std::env::set_var("RUST_BACKTRACE", "1")
    };

    // #[cfg(debug_assertions)]
    // deps::simple_logger::SimpleLogger::new().env().init().unwrap();

    #[cfg(any(target_os = "windows", target_os = "linux", target_os = "macos"))]
    run_with_iced::run()
}

pub fn init_tracing() {
    let fmt_layer = fmt::layer().compact();

    let filter_layer = EnvFilter::try_from_default_env()
        .or_else(|_| EnvFilter::try_new("info"))
        .unwrap();

    tracing_subscriber::registry()
        .with(filter_layer)
        .with(fmt_layer)
        .with(ErrorLayer::default())
        .init();
}
