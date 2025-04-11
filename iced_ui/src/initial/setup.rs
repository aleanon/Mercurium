use {
    iced::{
        widget::{self, text::LineHeight, Button},
        Element, Length, Task,
    },
    types::AppError,
    wallet::{wallet::Wallet, Unlocked},
    crate::{
        app::AppMessage,
        app::App,
    },
    super::{
        new_wallet::{self, NewWallet}, restore, restore_from_backup::{self, RestoreFromBackup}
    },
    inline_tweak::*,
};




#[derive(Clone)]
pub enum Message {
    SelectSetup,
    FromBackup,
    FromSeed,
    NewWallet,
    NewWalletMessage(new_wallet::Message),
    // RestoreFromSeedMessage(restore::Message),
    RestoreFromBackupMessage(restore_from_backup::Message),
    RestoreFromSeedMessage(restore::Message),
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
    // RestoreFromSeed(RestoreFromSeed),
    NewWallet(NewWallet),
    RestoreFromSeed(restore::RestoreFromSeed)
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
            Message::SelectSetup => *self = Self::SelectSetup,
            Message::NewWallet => *self = Self::NewWallet(NewWallet::new(wallet)),
            Message::FromSeed => *self = Self::RestoreFromSeed(restore::RestoreFromSeed::new(wallet)),
            
            Message::NewWalletMessage(new_wallet_message) => {
                if let Setup::NewWallet(new_wallet) = self {
                    return Ok(new_wallet.update(new_wallet_message, wallet).map(Message::NewWalletMessage));
                }
            }
            // Message::RestoreFromSeedMessage(message) => {
            //     if let Setup::RestoreFromSeed(restore_from_seed) = self {
            //         return restore_from_seed.update(message, app_data);
            //     }
            // }
            Message::RestoreFromSeedMessage(message) => {
                if let Setup::RestoreFromSeed(restore_from_seed) = self {
                    return Ok(restore_from_seed.update(message, wallet).map(Message::RestoreFromSeedMessage))

                }
            }
            Message::Error(_) => {/*Propagate*/}
            Message::WalletCreated(_) => {/*Propagate*/}
            _ => {}
        }
        Ok(Task::none())
    }
    
    #[inline_tweak::tweak_fn]
    pub fn view(&'a self, app: &'a App, wallet: &Wallet<wallet::Setup>) -> Element<'a, Message> {
        match self {
            Setup::SelectSetup => self.select_creation_view(),
            Setup::RestoreFromBackup(restore_from_backup) => restore_from_backup.view(app).map(Message::RestoreFromBackupMessage),
            Setup::NewWallet(new_wallet) => new_wallet.view(wallet).map(Message::NewWalletMessage),
            Setup::RestoreFromSeed(restore_from_seed) => restore_from_seed.view().map(|message| {
                match message {
                    restore::Message::SelectSetup => Message::SelectSetup,
                    _ => Message::RestoreFromSeedMessage(message)
                }
            }),
        }
    }

    #[inline_tweak::tweak_fn]
    fn select_creation_view(&self) -> Element<'_, Message> {
        let new_wallet =
            Self::creation_button("Create new wallet").on_press(Message::NewWallet);

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
    #[inline_tweak::tweak_fn] 
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
        .width(400)
        .height(100)
    }
}
