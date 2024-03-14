pub mod common_message;
pub mod login_message;
pub mod update_message;
pub mod app_view_message;
pub mod setup_message;



use iced::{Application, Command, Theme};

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
        match state.theme {
            Theme::CatppuccinFrappe => state.theme =  Theme::CatppuccinLatte,
            Theme::CatppuccinLatte => state.theme = Theme::CatppuccinMacchiato,
            Theme::CatppuccinMacchiato => state.theme = Theme::CatppuccinMocha,
            Theme::CatppuccinMocha => state.theme = Theme::Dark,
            Theme::Dark => state.theme = Theme::Dracula,
            Theme::Dracula => state.theme = Theme::GruvboxDark,
            Theme::GruvboxDark => state.theme = Theme::GruvboxLight,
            Theme::GruvboxLight => state.theme = Theme::KanagawaDragon,
            Theme::KanagawaDragon => state.theme = Theme::KanagawaLotus,
            Theme::KanagawaLotus => state.theme = Theme::KanagawaWave,
            Theme::KanagawaWave => state.theme = Theme::Moonfly,
            Theme::Moonfly => state.theme = Theme::Nightfly,
            Theme::Nightfly => state.theme = Theme::Nord,
            Theme::Nord => state.theme = Theme::Oxocarbon,
            Theme::Oxocarbon => state.theme = Theme::SolarizedDark,
            Theme::SolarizedDark => state.theme = Theme::SolarizedLight,
            Theme::SolarizedLight => state.theme = Theme::TokyoNight,
            Theme::TokyoNight => state.theme = Theme::TokyoNightLight,
            Theme::TokyoNightLight => state.theme = Theme::TokyoNightStorm,
            Theme::TokyoNightStorm => state.theme = Theme::Light,
            Theme::Light => state.theme = Theme::CatppuccinFrappe,
            _ => state.theme = Theme::Dark, 
        }

        Command::none()
    }

}

