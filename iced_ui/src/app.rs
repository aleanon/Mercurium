use std::collections::{BTreeSet, HashMap};

use debug_print::debug_println;
use iced::advanced::Application;
use iced::widget::image::Handle;
use iced::widget::{column, container, text};
use iced::{futures::channel::mpsc::Sender as MpscSender, Task};
use iced::{Length, Renderer};
use types::assets::{FungibleAsset, NonFungibleAsset};
use types::crypto::Password;
use types::notification::Notification;
use types::{
    theme::Theme, Account, AccountAddress, Action, AppError, AppSettings, Network, Resource,
    ResourceAddress,
};
// use iced_futures::futures::channel::mpsc::Sender as MpscSender;
// use iced_futures::futures::SinkExt;

use crate::common::Message;
use crate::error::errorscreen::ErrorMessage;
use crate::initial::setup::{self, Setup};
use crate::locked::loginscreen::{self, LoginScreen};
use crate::unlocked;
use crate::unlocked::app_view::AppView;
use crate::{task_response, tasks};

#[derive(Debug, Clone)]
pub enum AppMessage {
    Setup(setup::Message),
    Login(loginscreen::Message),
    AppView(unlocked::app_view::Message),
    Error(ErrorMessage),
    TaskResponse(task_response::Message),
    Common(Message),
    ToggleTheme,
    None,
}

use store::Db;

pub struct AppData {
    pub accounts: HashMap<AccountAddress, Account>,
    pub fungibles: HashMap<AccountAddress, BTreeSet<FungibleAsset>>,
    pub non_fungibles: HashMap<AccountAddress, BTreeSet<NonFungibleAsset>>,
    pub resources: HashMap<ResourceAddress, Resource>,
    pub resource_icons: HashMap<ResourceAddress, Handle>,
    pub settings: AppSettings,
    // Holds the sender for cloning into async tasks, there is a subscription
    // that listens on the receiver and produces Messages
    pub backend_sender: MpscSender<Action>,
    pub address_decoder: scrypto::address::AddressBech32Decoder,
    pub db: Option<Db>,
}

impl AppData {
    pub fn new(settings: AppSettings) -> Self {
        let address_decoder = scrypto::address::AddressBech32Decoder::new(&settings.network.into());
        Self {
            accounts: HashMap::new(),
            fungibles: HashMap::new(),
            non_fungibles: HashMap::new(),
            resources: HashMap::new(),
            resource_icons: HashMap::new(),
            settings,
            // Placeholder channel until the usable channel is returned from the subscription
            backend_sender: iced::futures::channel::mpsc::channel::<Action>(0).0,
            address_decoder,
            // Placeholder in-memory database until the actual database is received from the subscription
            // Placeholder in-memory database until the actual database is received from the subscription
            db: None,
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

impl Application for App {
    type Renderer = Renderer;
    type Message = AppMessage;
    type Executor = iced::executor::Default;
    type Theme = iced::Theme;
    type Flags = ();

    fn new(_flags: Self::Flags) -> (Self, iced::Task<Self::Message>) {
        let settings = handles::filesystem::app_settings::get_app_settings();

        let app_state =
            match handles::statics::initialize_statics::initialize_statics(Network::Mainnet) {
                Err(err) => AppState::Error(err.to_string()),
                Ok(_) => {
                    if Db::exists(settings.network) {
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

    //All panels have their own Message Enum, they are handled in their own module
    fn update(&mut self, message: AppMessage) -> Task<Self::Message> {
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
                loginscreen::Message::Login => match self.login() {
                    Ok(task) => return task,
                    Err(err) => self.handle_error(err),
                },
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

    fn view(&self) -> iced::Element<'_, Self::Message> {
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

    fn theme(&self) -> Self::Theme {
        self.app_data.settings.theme.into()
    }

    fn title(&self) -> String {
        String::from("RaVault")
    }
}

impl<'a> App {
    pub fn login(&mut self) -> Result<Task<AppMessage>, AppError> {
        let (is_initial, password) = match &self.app_state {
            AppState::Locked(loginscreen) => {
                (loginscreen.application_is_starting, &loginscreen.password)
            }
            _ => {
                return Err(AppError::Fatal(
                    "Called login when not in locked state".to_string(),
                ))
            }
        };

        let salt = handles::credentials::get_db_encryption_salt()?;
        let password_hash = password.derive_db_encryption_key_hash_from_salt(&salt);

        debug_println!("Initial login");

        let key = password.derive_db_encryption_key_from_salt(&salt);

        debug_println!("Key created");

        let db = Db::load(self.app_data.settings.network, &key)
            .map_err(|err| AppError::Fatal(err.to_string()))?;

        debug_println!("Database successfully loaded");

        let target_hash = db
            .get_db_password_hash()
            .map_err(|err| AppError::Fatal(err.to_string()))?;

        if password_hash == target_hash {
            self.app_state = AppState::Unlocked;
            if is_initial {
                return Ok(tasks::initial_login_tasks(self.app_data.settings.network));
            } else {
                return Ok(Task::none());
            }
        } else {
            let AppState::Locked(loginscreen) = &mut self.app_state else {
                return Err(AppError::Fatal(
                    "Called login when not in locked state".to_string(),
                ));
            };
            loginscreen.notification = "Incorrect password".to_string();
            return Err(AppError::NonFatal(Notification::None));
        }
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
