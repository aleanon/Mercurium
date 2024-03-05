pub mod loginscreen;
pub mod setup;
pub mod app_view;

use crate::{app::{App, State}, message::Message};


#[cfg_attr(feature="reload", no_mangle)]
pub fn view<'a>(app: &'a App) -> iced::Element<'a, Message> {
    match app.state {
        State::Initial(ref setup) => setup.view(app),
        State::Locked(ref loginscreen) => loginscreen.view(),
        State::Unlocked => app.appview.view(app),
        State::Error(ref _apperror) => app.appview.view(app),
    }
}