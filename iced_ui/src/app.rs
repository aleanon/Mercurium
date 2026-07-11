#[cfg(feature = "reload")]
use deps::hot_ice;

use deps::{
    debug_print::debug_println,
    hot_ice::hot_fn,
    iced::{
        self, Length, Task, Theme,
        widget::{container, text},
    },
};

use std::fmt::Debug;
use data_stores::AppDataDb;
use types::AppError;
use types::{Network, Notification, Theme as AppTheme};
use wallet::wallet::Wallet;
use wallet::{Env, Locked, Unlocked, WalletData};

use crate::common::Message;
use crate::initial::restore_from_seed;
use crate::initial::setup::{self, Setup};
use crate::locked::loginscreen::{self, LoginScreen};
use crate::unlocked;
use crate::unlocked::app_view::AppView;

#[derive(Clone)]
pub enum AppMessage {
    Setup(setup::Message),
    Login(loginscreen::Message),
    AppView(unlocked::app_view::Message),
    Error(AppError),
    Common(Message),
    ToggleTheme,
    Settings(crate::unlocked::settings::Action),
    None,
}

impl Debug for AppMessage {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "AppMessage")
    }
}

#[derive(Default)]
pub struct Preferences {
    pub theme: AppTheme,
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
    /// The live wallet profile (gateways, preferences, authorized dApps, factor-source metadata).
    pub profile: types::Profile,
    /// Injected capability bundle, threaded into every wallet this app constructs. Production uses
    /// `Env::production()`; iced_test `Preset`s inject a fake env for headless verification.
    pub env: Env,
}

impl App {
    #[hot_fn(feature = "reload")]
    pub fn new() -> (Self, Task<AppMessage>) {
        Self::new_with(Env::production())
    }

    /// Boot with an injected [`Env`]. Production calls this via [`App::new`] with
    /// `Env::production()`; iced_test `Preset` closures call it with a fake env to get a
    /// reproducible, offline app. See `.ai_docs/di_testability_plan.md`.
    pub fn new_with(env: Env) -> (Self, Task<AppMessage>) {
        let settings = wallet::Settings::load_from_disk_or_default(&env.paths);

        let app_state =
            match crate::bootstrap::initialize_statics(settings.network) {
                Err(err) => AppState::Error(err.to_string()),
                Ok(_) => {
                    if AppDataDb::exists(&env.paths, settings.network) {
                        AppState::Locked(
                            LoginScreen::new(true),
                            Wallet::new(
                                Locked::new(true),
                                WalletData::with_env(settings, env.clone()),
                            ),
                        )
                    } else {
                        AppState::Initial(
                            Setup::new(),
                            Wallet::new(
                                wallet::Setup::new(),
                                WalletData::with_env(settings, env.clone()),
                            ),
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
            profile: <data_stores::JsonProfileStore as data_stores::ProfileStore>::load(
                &data_stores::JsonProfileStore::new(env.paths.clone()),
            ),
            env,
        };

        (app, Task::none())
    }

    #[hot_fn(feature = "reload")]
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
            AppMessage::Settings(action) => self.apply_settings(action),
            AppMessage::Error(err) => self.handle_error(err),
            AppMessage::None => {}
        }
        task
    }

    #[hot_fn(feature = "reload")]
    pub fn view(&self) -> iced::Element<'_, AppMessage> {
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

    #[hot_fn(feature = "reload")]
    pub fn theme(&self) -> Option<Theme> {
        Some(to_iced_theme(self.preferences.theme))
    }

    #[hot_fn(feature = "reload")]
    pub fn title(&self) -> String {
        types::consts::APPLICATION_NAME.to_string()
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

    /// Applies a settings change to the live profile and persists it.
    fn apply_settings(&mut self, action: crate::unlocked::settings::Action) {
        use crate::unlocked::settings::Action;
        match action {
            Action::SwitchGateway(gateway) => self.profile.gateways.switch_to(gateway),
            Action::SetAppLockEnabled(enabled) => {
                let security = &mut self.profile.app_preferences.security;
                security.is_app_lock_enabled = enabled;
                // Disabling the lock discards the stored PIN hash; enabling only flips the flag,
                // leaving the UI to prompt for a PIN via `SetPin`.
                if !enabled {
                    security.app_lock = None;
                }
            }
            Action::SetPin(pin) => {
                match types::AppLock::new(&pin) {
                    Ok(lock) => {
                        let security = &mut self.profile.app_preferences.security;
                        security.app_lock = Some(lock);
                        security.is_app_lock_enabled = true;
                        self.notification = Notification::Info("PIN set".to_string());
                    }
                    Err(err) => {
                        self.notification = Notification::Warn(err.to_string());
                        return;
                    }
                }
            }
            Action::SetDeveloperMode(enabled) => {
                self.profile.app_preferences.security.is_developer_mode_enabled = enabled
            }
            Action::ExportBackup(password) => {
                let mut path = self.env.paths.config_directory();
                path.push("profile_backup.bin");
                match wallet::profile_backup::save_to_file(&self.profile, &password, &path) {
                    Ok(()) => {
                        self.notification = Notification::Info(format!(
                            "Encrypted backup saved to {}",
                            path.display()
                        ))
                    }
                    Err(err) => self.notification = Notification::Warn(err.to_string()),
                }
                return;
            }
            Action::ImportBackup(password) => {
                let mut path = self.env.paths.config_directory();
                path.push("profile_backup.bin");
                match wallet::profile_backup::load_from_file(&path, &password) {
                    Ok(profile) => {
                        self.profile = profile;
                        let _ = <data_stores::JsonProfileStore as data_stores::ProfileStore>::save(
                            &data_stores::JsonProfileStore::new(self.env.paths.clone()),
                            &self.profile,
                        );
                        self.notification = Notification::Info("Backup restored".to_string());
                    }
                    Err(err) => self.notification = Notification::Warn(err.to_string()),
                }
                return;
            }
        }
        if let Err(err) = <data_stores::JsonProfileStore as data_stores::ProfileStore>::save(
            &data_stores::JsonProfileStore::new(self.env.paths.clone()),
            &self.profile,
        ) {
            self.handle_error(err);
        }
    }

    fn toggle_theme(&mut self) {
        match self.preferences.theme {
            AppTheme::Dark => self.preferences.theme = AppTheme::Light,
            AppTheme::Light => self.preferences.theme = AppTheme::Dark,
            _ => self.preferences.theme = AppTheme::Dark,
        }
        // match self.preferences.theme {
        //     Theme::CatppuccinFrappe => self.preferences.theme = Theme::CatppuccinLatte,
        //     Theme::CatppuccinLatte => self.preferences.theme = Theme::CatppuccinMacchiato,
        //     Theme::CatppuccinMacchiato => self.preferences.theme = Theme::CatppuccinMocha,
        //     Theme::CatppuccinMocha => self.preferences.theme = Theme::Dark,
        //     Theme::Dark => self.preferences.theme = Theme::Dracula,
        //     Theme::Dracula => self.preferences.theme = Theme::GruvboxDark,
        //     Theme::GruvboxDark => self.preferences.theme = Theme::GruvboxLight,
        //     Theme::GruvboxLight => self.preferences.theme = Theme::KanagawaDragon,
        //     Theme::KanagawaDragon => self.preferences.theme = Theme::KanagawaLotus,
        //     Theme::KanagawaLotus => self.preferences.theme = Theme::KanagawaWave,
        //     Theme::KanagawaWave => self.preferences.theme = Theme::Moonfly,
        //     Theme::Moonfly => self.preferences.theme = Theme::Nightfly,
        //     Theme::Nightfly => self.preferences.theme = Theme::Nord,
        //     Theme::Nord => self.preferences.theme = Theme::Oxocarbon,
        //     Theme::Oxocarbon => self.preferences.theme = Theme::SolarizedDark,
        //     Theme::SolarizedDark => self.preferences.theme = Theme::SolarizedLight,
        //     Theme::SolarizedLight => self.preferences.theme = Theme::TokyoNight,
        //     Theme::TokyoNight => self.preferences.theme = Theme::TokyoNightLight,
        //     Theme::TokyoNightLight => self.preferences.theme = Theme::TokyoNightStorm,
        //     Theme::TokyoNightStorm => self.preferences.theme = Theme::Light,
        //     Theme::Light => self.preferences.theme = Theme::Ferra,
        //     Theme::Ferra => self.preferences.theme = Theme::CatppuccinFrappe,
        //     Theme::Custom => self.preferences.theme = Theme::Dark.into(),
        // }
    }

    pub fn current_network(&self) -> Network {
        match &self.app_state {
            AppState::Initial(_, wallet) => wallet.settings().network,
            AppState::Locked(_, wallet) => wallet.settings().network,
            AppState::Unlocked(wallet) => wallet.settings().network,
            AppState::Error(_) => Network::Mainnet,
        }
    }

    #[hot_fn(feature = "reload")]
    pub fn style(&self, theme: &iced::Theme) -> iced::theme::Style {
        let palette = theme.extended_palette();

        iced::theme::Style {
            background_color: palette.background.base.color,
            text_color: palette.background.base.text,
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

/// Maps the framework-neutral `types::Theme` onto iced's built-in theme.
/// Lives here (not in `types`) so the domain crate carries no iced dependency;
/// the orphan rule forbids a `From` impl here, hence a free function.
fn to_iced_theme(theme: AppTheme) -> Theme {
    match theme {
        AppTheme::Light => Theme::Light,
        AppTheme::Dark => Theme::Dark,
        AppTheme::Dracula => Theme::Dracula,
        AppTheme::Nord => Theme::Nord,
        AppTheme::SolarizedLight => Theme::SolarizedLight,
        AppTheme::SolarizedDark => Theme::SolarizedDark,
        AppTheme::GruvboxLight => Theme::GruvboxLight,
        AppTheme::GruvboxDark => Theme::GruvboxDark,
        AppTheme::CatppuccinLatte => Theme::CatppuccinLatte,
        AppTheme::CatppuccinFrappe => Theme::CatppuccinFrappe,
        AppTheme::CatppuccinMacchiato => Theme::CatppuccinMacchiato,
        AppTheme::CatppuccinMocha => Theme::CatppuccinMocha,
        AppTheme::TokyoNight => Theme::TokyoNight,
        AppTheme::TokyoNightStorm => Theme::TokyoNightStorm,
        AppTheme::TokyoNightLight => Theme::TokyoNightLight,
        AppTheme::KanagawaWave => Theme::KanagawaWave,
        AppTheme::KanagawaDragon => Theme::KanagawaDragon,
        AppTheme::KanagawaLotus => Theme::KanagawaLotus,
        AppTheme::Moonfly => Theme::Moonfly,
        AppTheme::Nightfly => Theme::Nightfly,
        AppTheme::Oxocarbon => Theme::Oxocarbon,
        AppTheme::Ferra => Theme::Ferra,
        AppTheme::Custom => Theme::Dark,
    }
}
