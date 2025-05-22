use deps::*;
use no_mangle_if_debug::no_mangle_if_debug;

use std::borrow::Cow;
use std::fmt::Debug;

use debug_print::debug_println;
use font_and_icons::images::WINDOW_LOGO;
use font_and_icons::BOOTSTRAP_FONT_BYTES;
use iced::time;
use iced::widget::{container, text};
use iced::{application, window, Length, Settings, Size};
use iced::{Subscription, Task};
use store::AppDataDb;
use types::AppError;
use types::{Network, Notification, Theme};
use wallet::wallet::Wallet;
use wallet::{Locked, Unlocked, WalletData};

use crate::common::Message;
use crate::initial::restore_from_seed;
use crate::initial::setup::{self, Setup};
use crate::locked::loginscreen::{self, LoginScreen};
use crate::unlocked;
use crate::unlocked::app_view::AppView;

//Reexport for hot reloading
pub use iced::Element;

#[derive(Clone)]
pub enum AppMessage {
    Setup(setup::Message),
    Login(loginscreen::Message),
    AppView(unlocked::app_view::Message),
    Error(AppError),
    Common(Message),
    ToggleTheme,
    None,
}

impl Debug for AppMessage {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "AppMessage")
    }
}

#[derive(Default)]
pub struct Preferences {
    pub theme: Theme,
}
// #[derive(Debug)]
pub enum AppState {
    Initial(Setup, Wallet<wallet::Setup>),
    Locked(LoginScreen, Wallet<Locked>),
    Unlocked(Wallet<Unlocked>),
    Error(String),
}

pub struct App {
    pub app_state: AppState,
    // pub app_data: AppData,
    pub appview: AppView,
    pub notification: Notification,
    pub preferences: Preferences,
}

impl App {
    pub fn new() -> (Self, Task<AppMessage>) {
        let settings = wallet::Settings::load_from_disk_or_default();

        let app_state =
            match handles::statics::initialize_statics::initialize_statics(settings.network) {
                Err(err) => AppState::Error(err.to_string()),
                Ok(_) => {
                    if AppDataDb::exists(settings.network) {
                        AppState::Locked(
                            LoginScreen::new(true),
                            Wallet::new(Locked::new(true), WalletData::new(settings)),
                        )
                    } else {
                        AppState::Initial(
                            Setup::new(),
                            Wallet::new(wallet::Setup::new(), WalletData::new(settings)),
                        )
                    }
                }
            };

        let app = App {
            app_state,
            // app_data: AppData::new(settings),
            appview: AppView::new(),
            notification: Notification::None,
            preferences: Preferences::default(),
        };

        (app, Task::none())
    }

    #[no_mangle_if_debug]
    pub fn update(&mut self, message: AppMessage) -> Task<AppMessage> {
        let mut task = Task::none();
        match message {
            AppMessage::Setup(message) => match message {
                setup::Message::RestoreFromSeedMessage(
                    restore_from_seed::Message::WalletCreated(wallet),
                ) => self.app_state = AppState::Unlocked(wallet),
                setup::Message::Error(err) => self.handle_error(err),
                message => {
                    if let AppState::Initial(setup, wallet) = &mut self.app_state {
                        match setup.update(message, wallet) {
                            Ok(task) => return task.map(AppMessage::Setup),
                            Err(err) => self.handle_error(err),
                        }
                    }
                }
            },
            AppMessage::Login(message) => {
                if let AppState::Locked(loginscreen, wallet) = &mut self.app_state {
                    if let loginscreen::Message::LoginSuccess(wallet, is_initial_login) = message {
                        if is_initial_login {
                            // task = external_tasks::initial_login_tasks(wallet.settings().network);
                        }
                        self.app_state = AppState::Unlocked(wallet);
                    } else {
                        task = loginscreen.update(message, wallet).map(AppMessage::Login);
                    }
                };
            }
            AppMessage::AppView(app_view_message) => {
                if let AppState::Unlocked(wallet) = &mut self.app_state {
                    return self.appview.update(app_view_message, wallet);
                }
            }
            AppMessage::Common(common_message) => return common_message.process(self),
            AppMessage::ToggleTheme => self.toggle_theme(),
            AppMessage::Error(err) => self.handle_error(err),
            AppMessage::None => {}
        }
        task
    }

    #[no_mangle_if_debug]
    pub fn view(&self) -> iced::Element<AppMessage> {
        match &self.app_state {
            AppState::Initial(setup, wallet) => setup.view(self, wallet).map(|message| {
                if let setup::Message::Error(err) = message {
                    AppMessage::Error(err)
                } else {
                    AppMessage::Setup(message)
                }
            }),
            AppState::Locked(loginscreen, _) => loginscreen.view().map(AppMessage::Login),
            AppState::Unlocked(wallet) => self.appview.view(wallet, self),
            AppState::Error(error) => container(text(error))
                .center_x(Length::Fill)
                .center_y(Length::Fill)
                .into(),
        }
    }

    // pub fn run() -> Result<(), iced::Error> {
    //     let icon = window::icon::from_file_data(
    //         WINDOW_LOGO,
    //         None,
    //     )
    //     .unwrap();

    //     let mut settings = Settings {
    //         antialiasing: true,
    //         ..Default::default()
    //     };
    //     settings.fonts.push(Cow::Borrowed(BOOTSTRAP_FONT_BYTES));

    //     let window_settings = window::Settings {
    //         min_size: Some(Size {
    //             height: 800.,
    //             width: 1000.,
    //         }),
    //         icon: Some(icon),
    //         ..Default::default()
    //     };

    //     let app_builder = application(App::new, App::update, App::view)
    //         .title(types::consts::APPLICATION_NAME)
    //         .settings(settings)
    //         .theme(|app|app.preferences.theme.into())
    //         .window(window_settings);

    //     #[cfg(debug_assertions)]
    //     app_builder
    //         .subscription(App::subscription)
    //         .run()?;

    //     #[cfg(not(debug_assertions))]
    //     app_builder.run_with(App::new)?;

    //     Ok(())
    // }

    #[cfg(debug_assertions)]
    pub fn subscription(&self) -> Subscription<AppMessage> {
        Subscription::batch([
            time::every(time::Duration::from_millis(500)).map(|_| AppMessage::None)
        ])
    }

    pub fn handle_error(&mut self, err: AppError) {
        debug_println!("Error: {err}");
        match err {
            AppError::Fatal(err) => self.app_state = AppState::Error(err),
            AppError::NonFatal(notification) => {
                self.notification = notification;
            }
            AppError::Ignore => {}
        }
    }

    fn toggle_theme(&mut self) {
        match self.preferences.theme {
            Theme::CatppuccinFrappe => self.preferences.theme = Theme::CatppuccinLatte,
            Theme::CatppuccinLatte => self.preferences.theme = Theme::CatppuccinMacchiato,
            Theme::CatppuccinMacchiato => self.preferences.theme = Theme::CatppuccinMocha,
            Theme::CatppuccinMocha => self.preferences.theme = Theme::Dark,
            Theme::Dark => self.preferences.theme = Theme::Dracula,
            Theme::Dracula => self.preferences.theme = Theme::GruvboxDark,
            Theme::GruvboxDark => self.preferences.theme = Theme::GruvboxLight,
            Theme::GruvboxLight => self.preferences.theme = Theme::KanagawaDragon,
            Theme::KanagawaDragon => self.preferences.theme = Theme::KanagawaLotus,
            Theme::KanagawaLotus => self.preferences.theme = Theme::KanagawaWave,
            Theme::KanagawaWave => self.preferences.theme = Theme::Moonfly,
            Theme::Moonfly => self.preferences.theme = Theme::Nightfly,
            Theme::Nightfly => self.preferences.theme = Theme::Nord,
            Theme::Nord => self.preferences.theme = Theme::Oxocarbon,
            Theme::Oxocarbon => self.preferences.theme = Theme::SolarizedDark,
            Theme::SolarizedDark => self.preferences.theme = Theme::SolarizedLight,
            Theme::SolarizedLight => self.preferences.theme = Theme::TokyoNight,
            Theme::TokyoNight => self.preferences.theme = Theme::TokyoNightLight,
            Theme::TokyoNightLight => self.preferences.theme = Theme::TokyoNightStorm,
            Theme::TokyoNightStorm => self.preferences.theme = Theme::Light,
            Theme::Light => self.preferences.theme = Theme::Ferra,
            Theme::Ferra => self.preferences.theme = Theme::CatppuccinFrappe,
            Theme::Custom => self.preferences.theme = Theme::Dark.into(),
        }
    }

    pub fn current_network(&self) -> Network {
        match &self.app_state {
            AppState::Initial(_, wallet) => wallet.settings().network,
            AppState::Locked(_, wallet) => wallet.settings().network,
            AppState::Unlocked(wallet) => wallet.settings().network,
            AppState::Error(_) => Network::Mainnet,
        }
    }

    pub fn wallet_data_mut(&mut self) -> Option<&mut WalletData> {
        match &mut self.app_state {
            AppState::Initial(_, _) => None,
            AppState::Locked(_, wallet) => Some(wallet.wallet_data_mut()),
            AppState::Unlocked(wallet) => Some(wallet.wallet_data_mut()),
            AppState::Error(_) => None,
        }
    }
}

// #[no_mangle_if_debug]
// pub fn update(state: &mut App, message: AppMessage) -> Task<AppMessage> {
//     let mut task = Task::none();
//     match message {
//         AppMessage::Setup(message) => match message {
//             setup::Message::RestoreFromSeedMessage(restore_from_seed::Message::WalletCreated(wallet)) => state.app_state = AppState::Unlocked(wallet),
//             setup::Message::Error(err) => state.handle_error(err),
//             message => {
//                 if let AppState::Initial(setup, wallet) = &mut state.app_state {
//                     match setup.update(message, wallet) {
//                         Ok(task) => return task.map(AppMessage::Setup),
//                         Err(err) => state.handle_error(err),
//                     }
//                 }
//             }
//         }
//         AppMessage::Login(message) => {
//             if let AppState::Locked(loginscreen, wallet) = &mut state.app_state {
//                 if let loginscreen::Message::LoginSuccess(wallet, is_initial_login) = message {
//                     if is_initial_login {
//                         // task = external_tasks::initial_login_tasks(wallet.settings().network);
//                     }
//                     state.app_state = AppState::Unlocked(wallet);
//                 } else {
//                     task = loginscreen.update(message, wallet).map(AppMessage::Login);
//                 }
//             };
//         },
//         AppMessage::AppView(app_view_message) => {
//             if let AppState::Unlocked(wallet) = &mut state.app_state {
//                 return state.appview.update(app_view_message, wallet);
//             }
//         }
//         AppMessage::Common(common_message) => return common_message.process(state),
//         AppMessage::ToggleTheme => state.toggle_theme(),
//         AppMessage::Error(err) => state.handle_error(err),
//         AppMessage::None => {}
//     }
//     task
// }

// #[no_mangle_if_debug]
// #[no_mangle]
// pub fn view(state: &App) -> Element<'_, AppMessage> {
//     match &state.app_state {
//         AppState::Initial(setup, wallet) => setup.view(state, wallet)
//             .map(|message|{
//                 if let setup::Message::Error(err) = message {
//                     AppMessage::Error(err)
//                 } else {
//                     AppMessage::Setup(message)
//                 }
//             }),
//         AppState::Locked(loginscreen, _) => loginscreen.view().map(AppMessage::Login),
//         AppState::Unlocked(wallet) => state.appview.view(wallet, state),
//         AppState::Error(error) => container(text(error))
//             .center_x(Length::Fill)
//             .center_y(Length::Fill)
//             .into(),
//     }
// }
