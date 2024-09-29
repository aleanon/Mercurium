use std::borrow::Cow;
use std::collections::{BTreeSet, HashMap};

use debug_print::debug_println;
use font_and_icons::images::WINDOW_LOGO;
use font_and_icons::BOOTSTRAP_FONT_BYTES;
use iced::widget::image::Handle;
use iced::widget::{container, text};
use iced::Task;
use iced::{application, window, Length, Settings, Size};
use store::AppDataDb;
use types::assets::{FungibleAsset, NonFungibleAsset};
use types::Notification;
use types::{
    address::{AccountAddress, ResourceAddress},
    Account, AppError, AppSettings, Network, Resource, Theme,
};
// use iced_futures::futures::channel::mpsc::Sender as MpscSender;
// use iced_futures::futures::SinkExt;

use crate::common::Message;
use crate::error::errorscreen::ErrorMessage;
use crate::external_task_response;
use crate::initial::setup::{self, Setup};
use crate::locked::loginscreen::{self, LoginScreen};
use crate::unlocked;
use crate::unlocked::app_view::AppView;

#[derive(Debug, Clone)]
pub enum AppMessage {
    Setup(setup::Message),
    Login(loginscreen::Message),
    AppView(unlocked::app_view::Message),
    Error(ErrorMessage),
    TaskResponse(external_task_response::Message),
    Common(Message),
    ToggleTheme,
    None,
}

#[derive(Debug, Clone)]
pub struct AppData {
    pub accounts: HashMap<AccountAddress, Account>,
    pub fungibles: HashMap<AccountAddress, BTreeSet<FungibleAsset>>,
    pub non_fungibles: HashMap<AccountAddress, BTreeSet<NonFungibleAsset>>,
    pub resources: HashMap<ResourceAddress, Resource>,
    pub resource_icons: HashMap<ResourceAddress, Handle>,
    pub settings: AppSettings,
}

impl<'a> AppData {
    pub fn new(settings: AppSettings) -> Self {
        Self {
            accounts: HashMap::new(),
            fungibles: HashMap::new(),
            non_fungibles: HashMap::new(),
            resources: HashMap::new(),
            resource_icons: HashMap::new(),
            settings,
        }
    }
}

#[derive(Debug)]
pub enum AppState {
    Initial(Setup),
    Locked(LoginScreen),
    Unlocked,
    Error(String),
}

pub struct App {
    version: [u8; 3],
    pub app_state: AppState,
    pub app_data: AppData,
    // Holds the gui unlocked state, not held in the AppState enum because we want to be able to return to last state on login
    pub appview: AppView,
    pub notification: Notification,
}

impl App {
    pub fn new() -> (Self, Task<AppMessage>) {
        let settings = handles::filesystem::app_settings::get_app_settings();

        let app_state =
            match handles::statics::initialize_statics::initialize_statics(Network::Mainnet) {
                Err(err) => AppState::Error(err.to_string()),
                Ok(_) => {
                    if AppDataDb::exists(settings.network) {
                        AppState::Locked(LoginScreen::new(true))
                    } else {
                        AppState::Initial(Setup::new())
                    }
                }
            };

        let app = App {
            version: [0, 0, 1],
            app_state,
            app_data: AppData::new(settings),
            appview: AppView::new(),
            notification: Notification::None,
        };

        (app, Task::none())
    }

    fn update(&mut self, message: AppMessage) -> Task<AppMessage> {
        match message {
            AppMessage::Setup(setup_message) => {
                if let AppState::Initial(setup) = &mut self.app_state {
                    match setup.update(setup_message, &mut self.app_data) {
                        Ok(task) => return task,
                        Err(err) => self.handle_error(err),
                    }
                }
            }
            AppMessage::Login(login_message) => match login_message {
                loginscreen::Message::LoginSuccess => {
                    let task = {
                        let AppState::Locked(loginscreen) = &mut self.app_state else {
                            unreachable!("Attempted to call login when not in a locked state")
                        };
                        loginscreen.update(login_message, &mut self.app_data)
                    };
                    self.app_state = AppState::Unlocked;
                    return task;
                }
                _ => {
                    if let AppState::Locked(ref mut loginscreen) = &mut self.app_state {
                        return loginscreen.update(login_message, &mut self.app_data);
                    }
                }
            },
            AppMessage::AppView(app_view_message) => {
                if let AppState::Unlocked = self.app_state {
                    return self.appview.update(app_view_message, &mut self.app_data);
                }
            }
            AppMessage::Common(common_message) => return common_message.process(self),
            AppMessage::TaskResponse(response_message) => {
                return self.process_task_response(response_message)
            }
            AppMessage::ToggleTheme => self.toggle_theme(),
            AppMessage::Error(error_message) => {}
            AppMessage::None => {}
        }
        Task::none()
    }

    fn view(&self) -> iced::Element<AppMessage> {
        match &self.app_state {
            AppState::Initial(setup) => setup.view(self),
            AppState::Locked(loginscreen) => loginscreen.view(),
            AppState::Unlocked => self.appview.view(&self.app_data),
            AppState::Error(error) => container(text(error))
                .center_x(Length::Fill)
                .center_y(Length::Fill)
                .into(),
        }
    }

    // fn subscription(&self) -> iced::Subscription<Self::Message> {
    //     Subscription::batch([crate::subscription::BackendWorker::backend_subscription()
    //         .map(|update| Message::Update(BackendMessage(update)))])
    // }

    pub fn run() -> Result<(), iced::Error> {
        let icon = window::icon::from_file_data(
            WINDOW_LOGO,
            Some(iced::advanced::graphics::image::image_rs::ImageFormat::Png),
        )
        .unwrap();

        let mut settings = Settings {
            antialiasing: false,
            ..Default::default()
        };
        settings.fonts.push(Cow::Borrowed(BOOTSTRAP_FONT_BYTES));

        let window_settings = window::Settings {
            min_size: Some(Size {
                height: 800.,
                width: 1000.,
            }),
            icon: Some(icon),
            ..Default::default()
        };

        application(types::consts::APPLICATION_NAME, App::update, App::view)
            .settings(settings)
            .window(window_settings)
            .run_with(|| App::new())?;

        Ok(())
    }

    pub fn handle_error(&mut self, err: AppError) {
        debug_println!("Error: {err}");
        match err {
            AppError::Fatal(err) => self.app_state = AppState::Error(err),
            AppError::NonFatal(err) => {
                self.notification = err;
            }
        }
    }

    fn toggle_theme(&mut self) {
        match self.app_data.settings.theme {
            Theme::CatppuccinFrappe => self.app_data.settings.theme = Theme::CatppuccinLatte,
            Theme::CatppuccinLatte => self.app_data.settings.theme = Theme::CatppuccinMacchiato,
            Theme::CatppuccinMacchiato => self.app_data.settings.theme = Theme::CatppuccinMocha,
            Theme::CatppuccinMocha => self.app_data.settings.theme = Theme::Dark,
            Theme::Dark => self.app_data.settings.theme = Theme::Dracula,
            Theme::Dracula => self.app_data.settings.theme = Theme::GruvboxDark,
            Theme::GruvboxDark => self.app_data.settings.theme = Theme::GruvboxLight,
            Theme::GruvboxLight => self.app_data.settings.theme = Theme::KanagawaDragon,
            Theme::KanagawaDragon => self.app_data.settings.theme = Theme::KanagawaLotus,
            Theme::KanagawaLotus => self.app_data.settings.theme = Theme::KanagawaWave,
            Theme::KanagawaWave => self.app_data.settings.theme = Theme::Moonfly,
            Theme::Moonfly => self.app_data.settings.theme = Theme::Nightfly,
            Theme::Nightfly => self.app_data.settings.theme = Theme::Nord,
            Theme::Nord => self.app_data.settings.theme = Theme::Oxocarbon,
            Theme::Oxocarbon => self.app_data.settings.theme = Theme::SolarizedDark,
            Theme::SolarizedDark => self.app_data.settings.theme = Theme::SolarizedLight,
            Theme::SolarizedLight => self.app_data.settings.theme = Theme::TokyoNight,
            Theme::TokyoNight => self.app_data.settings.theme = Theme::TokyoNightLight,
            Theme::TokyoNightLight => self.app_data.settings.theme = Theme::TokyoNightStorm,
            Theme::TokyoNightStorm => self.app_data.settings.theme = Theme::Light,
            Theme::Light => self.app_data.settings.theme = Theme::Ferra,
            Theme::Ferra => self.app_data.settings.theme = Theme::CatppuccinFrappe,
            Theme::Custom => self.app_data.settings.theme = Theme::Dark.into(),
        }
    }
}
