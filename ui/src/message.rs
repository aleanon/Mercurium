pub mod common_message;
pub mod login_message;
pub mod update_message;
pub mod app_view_message;
pub mod setup_message;



use iced::Command;

use crate::app::App;

use self::{app_view_message::AppViewMessage, common_message::CommonMessage, login_message::LoginMessage, setup_message::SetupMessage, update_message::UpdateMessage};

#[derive(Debug, Clone)]
pub enum Message {
    Setup(SetupMessage),
    AppView(AppViewMessage),
    Login(LoginMessage),
    Update(UpdateMessage),
    Common(CommonMessage),
    ToggleTheme,
    None,
}

impl<'a> Message {

    #[cfg_attr(feature="reload", no_mangle)]
    pub fn update(self, app: &'a mut App) -> Command<Message> {
        match self {
            Message::Common(common_message) => common_message.process(app),
            Message::ToggleTheme => Self::toggle_theme(app),
            Message::Update(update) => update.process(app),
            Message::AppView(app_view_message) => app_view_message.process(app),
            Message::Login(message) => message.process(app),
            Message::Setup(setup) => setup.process(app),
            Message::None => Command::none(),
        }
    }

    fn toggle_theme(state: &'a mut App) -> Command<Message> {
        state.darkmode = !state.darkmode;

        Command::none()
    }

}

