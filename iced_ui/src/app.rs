use std::collections::{BTreeSet, HashMap};

use iced::widget::image::Handle;
use types::{Account, Action, AppError, Fungible, NonFungible, ResourceAddress};
use debug_print::debug_println;
use iced::{Application, Command,  Theme, futures::channel::mpsc::Sender as MpscSender};
// use iced_futures::futures::channel::mpsc::Sender as MpscSender;
// use iced_futures::futures::SinkExt;


use crate::message::backend_message::BackendMessage;
use crate::message::Message;
use crate::view::app_view::AppView;
use crate::view::loginscreen::LoginScreen;
use crate::view::setup::Setup;

use store::Db;

#[derive(Debug)]
pub struct AppData {
    accounts: BTreeSet<Account>,
    fungibles: BTreeSet<Fungible>,
    non_fungibles: BTreeSet<NonFungible>,
    resource_icons: HashMap<ResourceAddress, Handle>
}

impl AppData {
    pub fn new() -> Self {
        Self {
            accounts: BTreeSet::new(),
            fungibles: BTreeSet::new(),
            non_fungibles: BTreeSet::new(),
            resource_icons: HashMap::new(),
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
    pub(crate) db: Option<Db>,
    pub(crate) action_tx: Option<MpscSender<Action>>,
    pub(crate) appview: AppView,
    pub(crate) theme: Theme,
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
            app_data: AppData::new(),
            db: None,
            action_tx: None,
            appview: AppView::new(),
            theme: Theme::Dark,
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
        crate::subscription::BackendWorker::backend_subscription()
            .map(|update| Message::Update(BackendMessage(update)))
        // BackEnd::backend_subscription().map(|update|Message::Update(UpdateMessage(update)))
    }

    fn theme(&self) -> Theme {
        self.theme.clone()
    }

    fn title(&self) -> String {
        String::from("RaVault")
    }

}

impl<'a> App {
    

    pub fn login(&mut self /*key: Key*/) -> Result<(), AppError> {
        match self.db {
            Some(_) => {
                // Self::load_accounts(&mut self.appview.center_panel,db);
                self.app_state = AppState::Unlocked;
                Ok(())
            }
            None => {
                match Db::load() {
                    Ok(db) => {
                        // Self::load_accounts(&mut self.appview.center_panel, &mut db);
                        self.db = Some(db);
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
    }
}
 