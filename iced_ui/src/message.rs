pub mod common_message;
pub mod login_message;
pub mod backend_message;
pub mod app_view_message;
pub mod setup_message;



use iced::{Application, Command, Theme};

use crate::app::{App, AppState};

use self::{app_view_message::AppViewMessage, common_message::CommonMessage, login_message::LoginMessage, setup_message::SetupMessage, backend_message::BackendMessage};

#[derive(Debug, Clone)]
pub enum Message {
    Setup(SetupMessage),
    AppView(AppViewMessage),
    Login(LoginMessage),
    Update(BackendMessage),
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
            Message::Login(login_message) => login_message.process(app),
            Message::Setup(setup_message) => Self::setup_message(setup_message, app),
            Message::None => Command::none(),
        }
    }

    fn setup_message(setup_message: SetupMessage, app: &'a mut App) -> Command<Message> {
        if let AppState::Initial(ref mut setup) = app.app_state {
            setup_message.process(setup, &mut app.app_data)
        } else {
            unreachable!()
        }
    }

    fn toggle_theme(state: &'a mut App) -> Command<Message> {
        match state.app_data.settings.theme {
            Theme::CatppuccinFrappe => state.app_data.settings.theme =  Theme::CatppuccinLatte,
            Theme::CatppuccinLatte => state.app_data.settings.theme = Theme::CatppuccinMacchiato,
            Theme::CatppuccinMacchiato => state.app_data.settings.theme = Theme::CatppuccinMocha,
            Theme::CatppuccinMocha => state.app_data.settings.theme = Theme::Dark,
            Theme::Dark => state.app_data.settings.theme = Theme::Dracula,
            Theme::Dracula => state.app_data.settings.theme = Theme::GruvboxDark,
            Theme::GruvboxDark => state.app_data.settings.theme = Theme::GruvboxLight,
            Theme::GruvboxLight => state.app_data.settings.theme = Theme::KanagawaDragon,
            Theme::KanagawaDragon => state.app_data.settings.theme = Theme::KanagawaLotus,
            Theme::KanagawaLotus => state.app_data.settings.theme = Theme::KanagawaWave,
            Theme::KanagawaWave => state.app_data.settings.theme = Theme::Moonfly,
            Theme::Moonfly => state.app_data.settings.theme = Theme::Nightfly,
            Theme::Nightfly => state.app_data.settings.theme = Theme::Nord,
            Theme::Nord => state.app_data.settings.theme = Theme::Oxocarbon,
            Theme::Oxocarbon => state.app_data.settings.theme = Theme::SolarizedDark,
            Theme::SolarizedDark => state.app_data.settings.theme = Theme::SolarizedLight,
            Theme::SolarizedLight => state.app_data.settings.theme = Theme::TokyoNight,
            Theme::TokyoNight => state.app_data.settings.theme = Theme::TokyoNightLight,
            Theme::TokyoNightLight => state.app_data.settings.theme = Theme::TokyoNightStorm,
            Theme::TokyoNightStorm => state.app_data.settings.theme = Theme::Light,
            Theme::Light => state.app_data.settings.theme = Theme::CatppuccinFrappe,
            _ => state.app_data.settings.theme = Theme::Dark, 
        }

        Command::none()
    }

}

