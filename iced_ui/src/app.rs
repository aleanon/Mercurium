use std::collections::{BTreeSet, HashMap};

use debug_print::debug_println;
use iced::widget::image::Handle;
use iced::{futures::channel::mpsc::Sender as MpscSender, Application, Command};
use types::assets::{FungibleAsset, NonFungibleAsset};
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
use crate::update::Update;

#[derive(Debug, Clone)]
pub enum AppMessage {
    Setup(setup::Message),
    AppView(unlocked::app_view::Message),
    Login(loginscreen::Message),
    Update(Update),
    Common(Message),
    Error(ErrorMessage),
    ToggleTheme,
    None,
}

use store::Db;

#[derive(Debug)]
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
    pub db: Db,
}

impl AppData {
    pub fn new(settings: AppSettings) -> Self {
        Self {
            accounts: HashMap::new(),
            fungibles: HashMap::new(),
            non_fungibles: HashMap::new(),
            resources: HashMap::new(),
            resource_icons: HashMap::new(),
            settings,
            // Placeholder channel until the usable channel is returned from the subscription
            backend_sender: iced::futures::channel::mpsc::channel::<Action>(0).0,
            // Placeholder in-memory database until the actual database is received from the subscription
            db: Db::new_in_memory(),
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
    pub(crate) app_state: AppState,
    pub(crate) app_data: AppData,
    // Holds the gui unlocked state, not held in the AppState enum because we want to be able to return to last state on login
    pub(crate) appview: AppView,
}

impl Application for App {
    type Message = AppMessage;
    type Executor = iced::executor::Default;
    type Theme = iced::Theme;
    type Flags = ();

    fn new(_flags: Self::Flags) -> (Self, iced::Command<Self::Message>) {
        let settings = handles::filesystem::app_settings::get_app_settings();

        let app_state =
            match handles::statics::initialize_statics::initialize_statics(Network::Mainnet) {
                Err(err) => AppState::Error(err.to_string()),
                Ok(_) => {
                    if Db::exists(settings.network) {
                        AppState::Locked(LoginScreen::new())
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
        };

        (app, Command::none())
    }

    //All panels have their own Message Enum, they are handled in their own module
    fn update(&mut self, message: AppMessage) -> Command<Self::Message> {
        let mut command = Command::none();
        match message {
            AppMessage::Setup(setup_message) => {
                if let AppState::Initial(setup) = &mut self.app_state {
                    command = setup.update(setup_message, &mut self.app_data);
                }
            }
            AppMessage::Login(login_message) => {
                if let AppState::Locked(ref mut loginscreen) = &mut self.app_state {
                    command = loginscreen.update(login_message, &mut self.app_data);
                }
            }
            AppMessage::AppView(app_view_message) => {
                if let AppState::Unlocked = self.app_state {
                    command = self.appview.update(app_view_message, &mut self.app_data);
                }
            }
            AppMessage::Common(common_message) => command = common_message.process(self),
            AppMessage::Update(update_message) => command = update_message.update(self),
            AppMessage::ToggleTheme => self.toggle_theme(),
            AppMessage::Error(error_message) => {}
            AppMessage::None => {}
        }
        command
    }

    fn view(&self) -> iced::Element<'_, Self::Message> {
        match &self.app_state {
            AppState::Initial(setup) => setup.view(self),
            AppState::Locked(loginscreen) => loginscreen.view(),
            AppState::Unlocked => self.appview.view(&self.app_data),
            AppState::Error(error) => self.appview.view(&self.app_data),
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
    pub fn login(&mut self /*key: Key*/) -> Result<(), AppError> {
        match Db::load(self.app_data.settings.network) {
            Ok(db) => {
                // Self::load_accounts(&mut self.appview.center_panel, &mut db);
                self.app_data.db = db;
                self.app_state = AppState::Unlocked;
                Ok(())
            }
            Err(err) => {
                debug_println!(
                    "{}:{}: Unable to load database, error: {}",
                    module_path!(),
                    line!(),
                    &err
                );

                Err(crate::app::AppError::Fatal(Box::new(err)))
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
            Theme::Light => self.app_data.settings.theme = Theme::CatppuccinFrappe,
            Theme::Custom => self.app_data.settings.theme = Theme::Dark.into(),
        }
    }
}
