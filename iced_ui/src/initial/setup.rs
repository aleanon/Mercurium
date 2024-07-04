use iced::{
    widget::{self, text::LineHeight, Button},
    Command, Element, Length,
};
use types::crypto::SeedPhrase;

use crate::{
    app::AppMessage,
    app::{App, AppData},
};

use super::{
    new_wallet::{self, NewWallet, NewWalletStage},
    restore_wallet::{self, RestoreWallet},
};

#[derive(Debug, Clone)]
pub enum Message {
    Back,
    Restore,
    FromSeed,
    NewWallet,
    NewWalletMessage(new_wallet::Message),
    RestoreWalletMessage(restore_wallet::Message),
}

impl Into<AppMessage> for Message {
    fn into(self) -> AppMessage {
        AppMessage::Setup(self)
    }
}

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
    pub fn update(&mut self, message: Message, app_data: &'a mut AppData) -> Command<AppMessage> {
        let mut command = Command::none();
        match message {
            Message::Back => self.back(),
            Message::NewWallet => {
                if let Setup::SelectCreation = self {
                    *self = Setup::NewWallet(NewWallet::new_with_mnemonic())
                }
            }
            Message::FromSeed => *self = Setup::NewWallet(NewWallet::new_without_mnemonic()),
            Message::NewWalletMessage(new_wallet_message) => {
                if let Setup::NewWallet(new_wallet) = self {
                    command = new_wallet.update(new_wallet_message, app_data);
                }
            }
            _ => {}
        }
        command
    }

    fn back(&mut self) {
        match self {
            Setup::NewWallet(new_wallet_state) => match new_wallet_state.stage {
                NewWalletStage::EnterPassword => *self = Setup::SelectCreation,
                NewWalletStage::VerifyPassword => {
                    new_wallet_state.stage = NewWalletStage::EnterPassword;
                    new_wallet_state.verify_password.clear();
                    new_wallet_state.notification = "";
                }
                NewWalletStage::EnterAccountName => {
                    new_wallet_state.stage = NewWalletStage::EnterPassword;
                    new_wallet_state.password.clear();
                    new_wallet_state.verify_password.clear();
                    new_wallet_state.notification = "";
                }
                NewWalletStage::EnterSeedPhrase => {
                    new_wallet_state.stage = NewWalletStage::EnterAccountName;
                    new_wallet_state.mnemonic = None;
                    new_wallet_state.notification = "";
                }
                NewWalletStage::ViewSeedPhrase => {
                    new_wallet_state.stage = NewWalletStage::EnterAccountName;
                    new_wallet_state.notification = "";
                }
                NewWalletStage::VerifySeedPhrase => {
                    new_wallet_state.stage = NewWalletStage::ViewSeedPhrase;
                    new_wallet_state.notification = "";
                    new_wallet_state.seed_phrase = SeedPhrase::new();
                }
            },
            _ => {}
        };
    }

    pub fn view(&self, app: &'a App) -> Element<'a, AppMessage> {
        let content: Element<'a, AppMessage> = match self {
            Self::SelectCreation => {
                let restore_from_backup =
                    Self::creation_button("Restore from backup").on_press(Message::Restore.into());

                let restore_from_seed =
                    Self::creation_button("Restore from seed").on_press(Message::FromSeed.into());

                let new_wallet =
                    Self::creation_button("Create new wallet").on_press(Message::NewWallet.into());

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
    ) -> Button<'a, AppMessage> {
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
