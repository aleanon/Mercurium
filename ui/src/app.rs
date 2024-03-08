use types::{Action, AppError};
use debug_print::debug_println;
use iced::{Application, Command,  Theme, futures::channel::mpsc::Sender as MpscSender};
// use iced_futures::futures::channel::mpsc::Sender as MpscSender;
// use iced_futures::futures::SinkExt;


use crate::message::update_message::UpdateMessage;
use crate::message::Message;
use crate::view::app_view::AppView;
use crate::view::loginscreen::LoginScreen;
use crate::view::setup::Setup;

use store::Db;


#[derive(Debug)]
pub enum State {
    Initial(Setup),
    Locked(LoginScreen),
    Unlocked,
    Error(AppError),
}

#[derive(Debug)]
pub struct App {
    pub(crate) state: State,
    pub(crate) db: Option<Db>,
    pub(crate) action_tx: Option<MpscSender<Action>>,
    pub(crate) appview: AppView,
    pub(crate) darkmode: bool,
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
                    state = State::Locked(LoginScreen::new());
                } else {
                    state = State::Initial(Setup::new());
                }
            }
            Err(err) => {
                state = State::Error(AppError::Fatal(Box::new(err)));
            }
        }

        let appstate = App {
            state,
            db: None,
            action_tx: None,
            appview: AppView::new(),
            darkmode: true,
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
            .map(|update| Message::Update(UpdateMessage(update)))
        // BackEnd::backend_subscription().map(|update|Message::Update(UpdateMessage(update)))
    }

    fn theme(&self) -> Theme {
        if self.darkmode {
            Theme::Dark
        } else {
            Theme::Light
        }
    }

    fn title(&self) -> String {
        String::from("RaVault")
    }
}

impl<'a> App {
    pub fn is_darkmode(&self) -> bool {
        self.darkmode
    }

    pub fn login(&mut self /*key: Key*/) -> Result<(), AppError> {
        match self.db {
            Some(_) => {
                // Self::load_accounts(&mut self.appview.center_panel,db);
                self.state = State::Unlocked;
                Ok(())
            }
            None => {
                match Db::load() {
                    Ok(db) => {
                        // Self::load_accounts(&mut self.appview.center_panel, &mut db);
                        self.db = Some(db);
                        self.state = State::Unlocked;
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
 