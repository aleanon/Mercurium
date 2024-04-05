use std::collections::{BTreeSet, HashMap};

use iced::widget::image::Handle;
use iced::Subscription;
use types::{Account, Action, AppError, Fungible, NonFungible, ResourceAddress};
use debug_print::debug_println;
use iced::{Application, Command,  Theme, futures::channel::mpsc::Sender as MpscSender};
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
}

impl AppSettings {
    pub fn new() -> Self {
        Self { 
            theme: Theme::Dark
        }
    }
}

#[derive(Debug)]
pub struct AppData {
    pub accounts: BTreeSet<Account>,
    pub fungibles: BTreeSet<Fungible>,
    pub non_fungibles: BTreeSet<NonFungible>,
    pub resource_icons: HashMap<ResourceAddress, Handle>,
    pub icons: Icons,
    pub settings: AppSettings,
    pub backend_sender: MpscSender<Action>,
    pub db: Db,
}

impl AppData {
    pub fn new(settings: Option<AppSettings>) -> Self {

        Self {
            accounts: BTreeSet::new(),
            fungibles: BTreeSet::new(),
            non_fungibles: BTreeSet::new(),
            resource_icons: HashMap::new(),
            icons: Icons::new(),
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
    pub(crate) app_state: AppState,
    pub(crate) app_data: AppData,
    // pub(crate) db: Option<Db>,
    // pub(crate) action_tx: Option<MpscSender<Action>>,
    // Holds the gui unlocked state
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

        let appstate = App {
            app_state: state,
            app_data: AppData::new(None),
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
        Subscription::batch([
            crate::subscription::BackendWorker::backend_subscription()
                .map(|update| Message::Update(BackendMessage(update))),
        ])
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
 