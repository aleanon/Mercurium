use std::collections::{BTreeSet, HashMap};

use debug_print::debug_println;
use iced::widget::image::Handle;
use iced::Subscription;
use iced::{futures::channel::mpsc::Sender as MpscSender, Application, Command, Theme};
use types::radix_request_client::{self, RadixDltRequestClient};
use types::{Account, Action, AppError, AppPath, Fungible, Network, NonFungible, ResourceAddress};
// use iced_futures::futures::channel::mpsc::Sender as MpscSender;
// use iced_futures::futures::SinkExt;

use crate::icons::Icons;
use crate::message::backend_message::BackendMessage;
use crate::message::Message;
use crate::view::app_view::AppView;
use crate::view::loginscreen::LoginScreen;
use crate::view::setup::Setup;

use store::Db;

#[derive(Debug)]
pub struct AppSettings {
    pub theme: Theme,
    pub network: Network,
}

impl AppSettings {
    pub fn new() -> Self {
        Self {
            theme: Theme::Dark,
            network: Network::Mainnet,
        }
    }
}

#[derive(Debug)]
pub struct AppData {
    pub app_path: AppPath,
    pub accounts: BTreeSet<Account>,
    pub fungibles: BTreeSet<Fungible>,
    pub non_fungibles: BTreeSet<NonFungible>,
    pub resource_icons: HashMap<ResourceAddress, Handle>,
    pub icons: Icons,
    pub radix_dlt_request_client: RadixDltRequestClient,
    pub settings: AppSettings,
    pub backend_sender: MpscSender<Action>,
    pub db: Db,
}

impl AppData {
    pub fn new(settings: Option<AppSettings>, app_path: AppPath) -> Self {
        let radix_request_client = RadixDltRequestClient::new().unwrap();

        Self {
            app_path,
            accounts: BTreeSet::new(),
            fungibles: BTreeSet::new(),
            non_fungibles: BTreeSet::new(),
            resource_icons: HashMap::new(),
            icons: Icons::new(),
            radix_dlt_request_client: radix_request_client,
            settings: settings.unwrap_or(AppSettings::new()),
            // Placeholder channel until the usable channel is returned from the subscription
            backend_sender: iced::futures::channel::mpsc::channel::<Action>(0).0,
            // Placeholder in-memory database until the actual database is received from the subscription
            db: Db::placeholder(),
        }
    }
}

#[derive(Debug)]
pub enum AppState {
    Initial(Setup),
    Locked(LoginScreen),
    Unlocked,
    Error(AppError),
}

#[derive(Debug)]
pub struct App {
    version: [u8; 3],
    pub(crate) app_state: AppState,
    pub(crate) app_data: AppData,
    // pub(crate) db: Option<Db>,
    // pub(crate) action_tx: Option<MpscSender<Action>>,
    // Holds the gui unlocked state, not held in the AppState enum because we want to be able to return to last state on login
    pub(crate) appview: AppView,
    // pub(crate) theme: Theme,
}

impl Application for App {
    type Message = Message;
    type Executor = iced::executor::Default;
    type Theme = Theme;
    type Flags = ();

    fn new(_flags: Self::Flags) -> (Self, iced::Command<Message>) {
        let state;

        match Db::exits() {
            Ok(exists) => {
                if exists {
                    state = AppState::Locked(LoginScreen::new());
                } else {
                    state = AppState::Initial(Setup::new());
                }
            }
            Err(err) => {
                state = AppState::Error(AppError::Fatal(Box::new(err)));
            }
        }

        let app_path = AppPath::new().expect("Unable to establish app directory");

        let appstate = App {
            version: [0, 0, 1],
            app_state: state,
            app_data: AppData::new(None, app_path),
            // db: None,
            // action_tx: None,
            appview: AppView::new(),
            // theme: Theme::Dark,
        };

        (appstate, Command::none())
    }

    //All panels have their own Message Enum, they are handled in their own module
    fn update(&mut self, message: Message) -> Command<Message> {
        message.update(self)
    }

    fn view(&self) -> iced::Element<'_, Self::Message> {
        super::view::view(self)
    }

    fn subscription(&self) -> iced::Subscription<Self::Message> {
        Subscription::batch([crate::subscription::BackendWorker::backend_subscription()
            .map(|update| Message::Update(BackendMessage(update)))])
    }

    fn theme(&self) -> Theme {
        self.app_data.settings.theme.clone()
    }

    fn title(&self) -> String {
        String::from("RaVault")
    }
}

impl<'a> App {
    pub fn login(&mut self /*key: Key*/) -> Result<(), AppError> {
        // match self.db {
        //     Some(_) => {
        //         // Self::load_accounts(&mut self.appview.center_panel,db);
        //         self.app_state = AppState::Unlocked;
        //         Ok(())
        //     }
        //     None => {
        //         match Db::load() {
        //             Ok(db) => {
        //                 // Self::load_accounts(&mut self.appview.center_panel, &mut db);
        //                 self.db = Some(db);
        //                 self.app_state = AppState::Unlocked;
        //                 Ok(())
        //             }
        //             Err(err) => {
        //                 debug_println!(
        //                     "{}:{}: Unable to load database, error: {}",
        //                     module_path!(),
        //                     line!(),
        //                     &err
        //                 );

        //                 Err(crate::app::AppError::Fatal(Box::new(err)))
        //             }
        //         }
        //     }
        // }

        match Db::load() {
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
