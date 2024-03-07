#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use anyhow::Result;
use iced::{Application, Settings};

#[cfg(not(feature="reload"))]
use ui::App;

#[cfg(feature="reload")]
use hot_lib;

#[cfg(feature="reload")]
#[hot_lib_reloader::hot_module(dylib = "../ui")]
mod hot_lib {
    use ui::App;

    hot_functions_from_file!("crates/ui/src/view.rs");
}





fn main() -> Result<()> {
    // let (to_frontend_tx, from_backend_rx) = Mpsc::channel();
    // let (to_backend_tx, from_frontend_rx) = Mpsc::channel();

    // let sender = std::thread::spawn({
    //     move || {
    //         let backend_runtime = Builder::new_current_thread()
    //             .enable_all()
    //             .build()
    //             .expect("Unable to create thread");

    //         backend_runtime
    //             .block_on(async {
    //                 let mut backend = BackEnd::new(to_frontend_tx, from_frontend_rx)
    //                     .expect("Unable to create backend");
    //                 match backend.load().await {
    //                     Ok(_) => anyhow::Ok(()),
    //                     Err(_) => return Err(anyhow::Error::msg("failed to load backend")),
    //                 }
    //             })
    //             .unwrap();
    //     }
    // });

    let settings: iced::Settings<()> = Settings {
        flags: (),
        antialiasing: false,
        window: iced::window::Settings {
            min_size: Some(iced::Size { width: 1000., height: 700. }),
            position: iced::window::Position::Centered,
            ..Default::default()
        },
        default_font: Default::default(),
        id: Some(String::from("ravault")),
        ..Default::default()
    };
    App::run(settings)?;

    // #[cfg(debug_assertions)]
    // {
    //     println!("Application closed, press ENTER to close window");
    //     let mut str = String::new();
    //     loop {
    //         let input = std::io::stdin().read_line(&mut str);
    //         match input {
    //             _ => break,
    //         }
    //     }
    // }

    Ok(())
}
