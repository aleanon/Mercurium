use iced::{
    widget::{self, text::LineHeight, Button},
    Element, Length, Task,
};
use types::{crypto::SeedPhrase, AppError};

use crate::{
    app::AppMessage,
    app::{App, AppData},
};

use super::{
    new_wallet::{self, NewWallet, NewWalletStage}, restore_from_seed::{self, RestoreFromSeed}, restore_wallet::{self, RestoreFromBackup}
};

#[derive(Debug, Clone)]
pub enum Message {
    Back,
    FromBackup,
    FromSeed,
    NewWallet,
    NewWalletMessage(new_wallet::Message),
    RestoreFromSeedMessage(restore_from_seed::Message),
    RestoreWalletMessage(restore_wallet::Message),
}

impl Into<AppMessage> for Message {
    fn into(self) -> AppMessage {
        AppMessage::Setup(self)
    }
}

#[derive(Debug)]
pub enum Setup {
    SelectSetup,
    RestoreFromBackup(RestoreFromBackup),
    RestoreFromSeed(RestoreFromSeed),
    NewWallet(NewWallet),
}

impl<'a> Setup {
    pub fn new() -> Self {
        Self::SelectSetup
    }

    pub fn update(
        &mut self,
        message: Message,
        app_data: &'a mut AppData,
    ) -> Result<Task<AppMessage>, AppError> {
        match message {
            Message::Back => self.back(),
            Message::NewWallet => {
                if let Setup::SelectSetup = self {
                    *self = Setup::NewWallet(NewWallet::new_with_mnemonic())
                }
            }
            Message::FromSeed => {
                *self = Setup::RestoreFromSeed(RestoreFromSeed::new())
            }
            Message::NewWalletMessage(new_wallet_message) => {
                if let Setup::NewWallet(new_wallet) = self {
                    return new_wallet.update(new_wallet_message, app_data);
                }
            }
            Message::RestoreFromSeedMessage(message) => {
                if let Setup::RestoreFromSeed(restore_from_seed) = self {
                    return restore_from_seed.update(message, app_data);
                }
            }
            _ => {}
        }
        Ok(Task::none())
    }

    fn back(&mut self) {
        match self {
            Setup::NewWallet(new_wallet_state) => match new_wallet_state.stage {
                NewWalletStage::EnterPassword => *self = Setup::SelectSetup,
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
            Setup::RestoreFromSeed(_) => *self = Self::SelectSetup,
            _ => {}
        };
    }

    pub fn view(&'a self, app: &'a App) -> Element<'a, AppMessage> {
        let content: Element<'a, AppMessage> = match self {
            Setup::SelectSetup => self.select_creation_view(),
            Setup::RestoreFromBackup(restore_from_backup) => restore_from_backup.view(app),
            Setup::NewWallet(new_wallet) => new_wallet.view(app),
            Setup::RestoreFromSeed(restore_from_seed) => restore_from_seed.view(app),
        };

        widget::container(content)
            .center_x(Length::Fill)
            .center_y(Length::Fill)
            .into()
    }

    fn select_creation_view(&self) -> Element<'_, AppMessage> {
        let restore_from_backup =
            Self::creation_button("Restore from backup").on_press(Message::FromBackup.into());

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

    //todo: replace space with image from handle
    pub fn creation_button(
        text: &'a str, /*handle: iced::widget::image::Handle*/
    ) -> Button<'a, AppMessage> {
        Button::new(widget::column![
            widget::text(text)
                .size(20)
                .line_height(LineHeight::Relative(2.))
                .align_x(iced::alignment::Horizontal::Center)
                .align_y(iced::alignment::Vertical::Center)
                .width(Length::Fill)
                .height(Length::Shrink),
            widget::Space::new(Length::Fill, Length::Fill)
        ])
        .width(400)
        .height(100)
    }
}
