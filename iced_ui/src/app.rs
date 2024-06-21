use std::collections::{BTreeSet, HashMap};
use std::fs::File;
use std::io::{BufReader, Read};

use debug_print::debug_println;
use iced::widget::image::Handle;
use iced::{futures::channel::mpsc::Sender as MpscSender, Application, Command, Theme};
use types::app_error::ErrorString;
use types::app_path::AppPath;
use types::assets::{FungibleAsset, NonFungibleAsset};
use types::{
    Account, AccountAddress, Action, AppError, AppSettings, Network, Resource, ResourceAddress,
};
// use iced_futures::futures::channel::mpsc::Sender as MpscSender;
// use iced_futures::futures::SinkExt;

use crate::message::{ErrorMessage, Message};
use crate::view::app_view::AppView;
use crate::view::loginscreen::LoginScreen;
use crate::view::setup::Setup;

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
    type Message = Message;
    type Executor = iced::executor::Default;
    type Theme = Theme;
    type Flags = ();

    fn new(_flags: Self::Flags) -> (Self, iced::Command<Message>) {
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
    fn update(&mut self, message: Message) -> Command<Message> {
        message.update(self)
    }

    fn view(&self) -> iced::Element<'_, Self::Message> {
        super::view::view(self)
    }

    // fn subscription(&self) -> iced::Subscription<Self::Message> {
    //     Subscription::batch([crate::subscription::BackendWorker::backend_subscription()
    //         .map(|update| Message::Update(BackendMessage(update)))])
    // }

    fn theme(&self) -> Theme {
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
}
