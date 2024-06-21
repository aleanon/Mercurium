pub mod new_wallet;
pub mod restore_wallet;

use iced::{
    widget::{self, text::LineHeight, Button},
    Element, Length,
};

use crate::{
    app::App,
    message::{setup_message::SetupMessage, Message},
};

use self::{new_wallet::NewWallet, restore_wallet::RestoreWallet};

#[derive(Debug)]
pub enum Setup {
    SelectCreation,
    RestoreWallet(RestoreWallet),
    NewWallet(NewWallet),
}

impl<'a> Setup {
    pub fn new() -> Self {
        Self::SelectCreation
    }
}

impl<'a> Setup {
    pub fn view(&self, app: &'a App) -> Element<'a, Message> {
        let content: Element<'a, Message> = match self {
            Self::SelectCreation => {
                let restore_from_backup = Self::creation_button("Restore from backup")
                    .on_press(SetupMessage::Restore.into());

                let restore_from_seed = Self::creation_button("Restore from seed")
                    .on_press(SetupMessage::FromSeed.into());

                let new_wallet = Self::creation_button("Create new wallet")
                    .on_press(SetupMessage::NewWallet.into());

                widget::column![restore_from_backup, restore_from_seed, new_wallet]
                    .width(Length::Shrink)
                    .height(Length::Shrink)
                    .spacing(40)
                    .into()
            }
            Self::RestoreWallet(restore) => restore.view(app),
            Self::NewWallet(new_wallet) => new_wallet.view(app),
        };

        widget::container(content)
            .width(Length::Fill)
            .height(Length::Fill)
            .center_x()
            .center_y()
            .into()
    }

    //todo: replace space with image from handle
    pub fn creation_button(
        text: &str, /*handle: iced::widget::image::Handle*/
    ) -> Button<'a, Message> {
        Button::new(widget::column![
            widget::text(text)
                .size(20)
                .line_height(LineHeight::Relative(2.))
                .horizontal_alignment(iced::alignment::Horizontal::Center)
                .vertical_alignment(iced::alignment::Vertical::Center)
                .width(Length::Fill)
                .height(Length::Shrink),
            widget::Space::new(Length::Fill, Length::Fill)
        ])
        .width(400)
        .height(100)
    }
}
