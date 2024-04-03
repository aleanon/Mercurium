pub mod loginscreen;
pub mod setup;
pub mod app_view;

use crate::{app::{App, AppState}, message::Message};


#[cfg_attr(feature="reload", no_mangle)]
pub fn view<'a>(app: &'a App) -> iced::Element<'a, Message> {
    match app.app_state {
        AppState::Initial(ref setup) => setup.view(app),
        AppState::Locked(ref loginscreen) => loginscreen.view(),
        AppState::Unlocked => app.appview.view(app),
        AppState::Error(ref _apperror) => app.appview.view(app),
    }
}