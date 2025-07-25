use deps::*;

use crate::styles;

use super::restore_from_seed::{Action, RestoreFromSeed};

use {
    super::{
        new_wallet::{self, NewWallet},
        restore_from_backup::{self, RestoreFromBackup},
        restore_from_seed,
    },
    crate::{app::App, app::AppMessage},
    iced::{
        widget::{self, text::LineHeight, Button},
        Element, Length, Task,
    },
    types::AppError,
    wallet::{wallet::Wallet, Unlocked},
};

#[derive(Clone)]
pub enum Message {
    SelectSetup,
    FromBackup,
    FromSeed,
    NewWallet,
    NewWalletMessage(new_wallet::Message),
    RestoreFromBackupMessage(restore_from_backup::Message),
    RestoreFromSeedMessage(restore_from_seed::Message),
    WalletCreated(Wallet<Unlocked>),
    Error(AppError),
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
    NewWallet(NewWallet),
    RestoreFromSeed(RestoreFromSeed),
}

impl<'a> Setup {
    pub fn new() -> Self {
        Self::SelectSetup
    }

    pub fn update(
        &mut self,
        message: Message,
        wallet: &'a mut Wallet<wallet::Setup>,
    ) -> Result<Task<Message>, AppError> {
        match message {
            Message::SelectSetup => {
                *self = Self::SelectSetup;
                wallet.reset();
            }
            Message::NewWallet => *self = Self::NewWallet(NewWallet::new(wallet)),
            Message::FromSeed => {
                *self = Self::RestoreFromSeed(restore_from_seed::RestoreFromSeed::new(wallet))
            }

            Message::NewWalletMessage(new_wallet_message) => {
                if let new_wallet::Message::SetupSelection = new_wallet_message {
                    *self = Self::SelectSetup
                } else if let Setup::NewWallet(new_wallet) = self {
                    return Ok(new_wallet
                        .update(new_wallet_message, wallet)
                        .map(Message::NewWalletMessage));
                }
            }
            // Message::RestoreFromSeedMessage(message) => {
            //     if let Setup::RestoreFromSeed(restore_from_seed) = self {
            //         return restore_from_seed.update(message, app_data);
            //     }
            // }
            Message::RestoreFromSeedMessage(message) => {
                if let Setup::RestoreFromSeed(restore_from_seed) = self {
                    match restore_from_seed.update(message, wallet) {
                        restore_from_seed::Action::SetupSelection => {
                            *self = Self::SelectSetup;
                            wallet.reset();
                        }
                        restore_from_seed::Action::Task(task) => {
                            return Ok(task.map(Message::RestoreFromSeedMessage))
                        }
                        Action::None => {}
                    }
                }
            }
            Message::Error(_) => { /*Propagate*/ }
            Message::WalletCreated(_) => { /*Propagate*/ }
            _ => {}
        }
        Ok(Task::none())
    }

    pub fn view(&'a self, app: &'a App, wallet: &Wallet<wallet::Setup>) -> Element<'a, Message> {
        match self {
            Setup::SelectSetup => self.select_creation_view(),
            Setup::RestoreFromBackup(restore_from_backup) => restore_from_backup
                .view(app)
                .map(Message::RestoreFromBackupMessage),
            Setup::NewWallet(new_wallet) => new_wallet.view(wallet).map(|m| match m {
                new_wallet::Message::SetupSelection => Message::SelectSetup,
                message => Message::NewWalletMessage(message),
            }),
            Setup::RestoreFromSeed(restore_from_seed) => restore_from_seed
                .view()
                .map(Message::RestoreFromSeedMessage),
        }
    }

    fn select_creation_view(&self) -> Element<'_, Message> {
        let new_wallet = Self::creation_button("Create new wallet").on_press(Message::NewWallet);

        let restore_from_backup =
            Self::creation_button("Restore from backup").on_press(Message::FromBackup);

        let restore_from_seed =
            Self::creation_button("Restore from seed").on_press(Message::FromSeed);

        let content = widget::column![new_wallet, restore_from_backup, restore_from_seed]
            .width(Length::Shrink)
            .height(Length::Shrink)
            .spacing(40);

        widget::container(content)
            .center_x(Length::Fill)
            .center_y(Length::Fill)
            .into()
    }

    //todo: replace space with svg
    pub fn creation_button<Message: Clone + 'a>(
        text: &'a str, /*handle: iced::widget::image::Handle*/
    ) -> Button<'a, Message> {
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
        .style(styles::button::setup_selection)
        .width(400)
        .height(100)
    }
}
