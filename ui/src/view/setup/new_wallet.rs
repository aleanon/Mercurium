use bip39::{Language, Mnemonic, MnemonicType};
use iced::{
    widget::{self, text_input::Id, Button, Column, Row},
    Element, Length,
};
use iced::widget::text::LineHeight;
use types::crypto::{Password, SeedPhrase};
use zeroize::Zeroize;

use crate::{app::App, message::{setup_message::new_wallet_update::WalletMessage, Message}};


#[derive(Debug)]
pub enum NewWalletStage {
    EnterPassword,
    VerifyPassword,
    EnterAccountName,
    EnterSeedPhrase,
    ViewSeedPhrase,
    VerifySeedPhrase,
}


#[derive(Debug)]
pub struct NewWallet {
    pub(crate) stage: NewWalletStage,
    pub(crate) notification: &'static str,
    pub(crate) password: Password,
    pub(crate) verify_password: Password,
    pub(crate) account_name: String,
    pub(crate) mnemonic: Option<Mnemonic>,
    pub(crate) seed_phrase: SeedPhrase,
}

impl NewWallet {
    pub fn new_with_mnemonic() -> Self {
        Self {
            stage: NewWalletStage::EnterPassword,
            notification: "",
            password: Password::new(),
            verify_password: Password::new(),
            account_name: String::new(),
            mnemonic: Some(Mnemonic::new(MnemonicType::Words24, Language::English)),
            seed_phrase: SeedPhrase::new(),
        }
    }

    pub fn new_without_mnemonic() -> Self {
        Self {
            stage: NewWalletStage::EnterPassword,
            notification: "",
            password: Password::new(),
            verify_password: Password::new(),
            account_name: String::new(),
            mnemonic: None,
            seed_phrase: SeedPhrase::new(),
        }
    }
}

impl<'a> NewWallet {
    pub fn view(&self, _app: &'a App) -> Element<'a, Message> {
        let content = match self.stage {
            NewWalletStage::EnterPassword => self.enter_password_pane(),
            NewWalletStage::VerifyPassword => self.verify_password_pane(),
            NewWalletStage::EnterAccountName => self.account_name_pane(),
            NewWalletStage::EnterSeedPhrase => self.enter_seed_phrase_pane(),
            NewWalletStage::ViewSeedPhrase => self.view_seed_phrase(),
            NewWalletStage::VerifySeedPhrase => self.enter_seed_phrase_pane(),
        };

        widget::container(content)
            .width(660)
            .height(700)
            .center_x()
            .center_y()
            .into()
    }

    fn enter_password_pane(&self) -> Column<'a, Message> {
        let notification = Self::notification_field(self.notification);

        let password_input = Self::text_input_field("Enter Password", &self.password.as_str())
            .on_submit(WalletMessage::SubmitPassword.into())
            .on_paste(|input| WalletMessage::UpdatePassword(input).into())
            .on_input(|input| WalletMessage::UpdatePassword(input).into())
            .secure(true);

        let back = Self::nav_button("Back").on_press(WalletMessage::Back.into());

        let next = Self::nav_button("Next").on_press(WalletMessage::SubmitPassword.into());

        let nav = Self::nav_row(back, next);

        widget::column![notification, password_input, nav]
            .align_items(iced::Alignment::Center)
            .width(Length::Shrink)
            .height(Length::Shrink)
            .spacing(50)
    }

    fn verify_password_pane(&self) -> Column<'a, Message> {
        let notification = Self::notification_field(self.notification);

        let password_input =
            Self::text_input_field("Verify Password", &self.verify_password.as_str())
                .on_submit(WalletMessage::VerifiPassword.into())
                .on_paste(|input| WalletMessage::UpdateVerificationPassword(input).into())
                .on_input(|input| WalletMessage::UpdateVerificationPassword(input).into())
                .secure(true);

        let back = Self::nav_button("Back").on_press(WalletMessage::Back.into());

        let next = Self::nav_button("Next").on_press(WalletMessage::VerifiPassword.into());

        let nav = Self::nav_row(back, next);

        widget::column![notification, password_input, nav]
            .align_items(iced::Alignment::Center)
            .width(Length::Shrink)
            .height(Length::Shrink)
            .spacing(50)
    }

    fn account_name_pane(&self) -> Column<'a, Message> {
        let notification = Self::notification_field(self.notification);

        let account_name = Self::text_input_field("Enter account name", &self.account_name)
            .on_submit(WalletMessage::SubmitAccName.into())
            .on_input(|input| WalletMessage::UpdateAccName(input).into());

        let back = Self::nav_button("Back").on_press(WalletMessage::Back.into());

        let next = Self::nav_button("Next").on_press(WalletMessage::SubmitAccName.into());

        let nav = Self::nav_row(back, next);

        widget::column![notification, account_name, nav]
            .align_items(iced::Alignment::Center)
            .width(Length::Shrink)
            .height(Length::Shrink)
            .spacing(50)
    }

    fn view_seed_phrase(&self) -> Column<'a, Message> {
        let mut seed = widget::column![]
            .width(Length::Shrink)
            .height(Length::Shrink)
            .spacing(20);
        let mut row = widget::row![]
            .width(Length::Shrink)
            .height(Length::Shrink)
            .spacing(20);

        let seed_phrase = match self.mnemonic {
            Some(ref mnemonic) => mnemonic.phrase(),
            None => "",
        };

        for (i, word) in seed_phrase.split_ascii_whitespace().enumerate() {
            if (i) % 4 == 0 && i != 0 {
                seed = seed.push(row);
                row = widget::row![]
                    .width(Length::Shrink)
                    .height(Length::Shrink)
                    .spacing(20);
            }

            let text_field = Self::seed_word_field("", word).on_input(|mut string| {
                string.zeroize();
                Message::None
            });

            row = row.push(text_field);
        }
        seed = seed.push(row);

        let back = Self::nav_button("Back").on_press(WalletMessage::Back.into());

        let next = Self::nav_button("Next").on_press(WalletMessage::VerifySeedPhrase.into());

        let nav = Self::nav_row(back, next);

        widget::column![seed, nav]
            .width(Length::Shrink)
            .height(Length::Shrink)
            .spacing(50)
    }

    fn enter_seed_phrase_pane(&self) -> Column<'a, Message> {
        let input_seed = self.input_seed();

        let back = Self::nav_button("Back").on_press(WalletMessage::Back.into());

        let next = Self::nav_button("Next").on_press(WalletMessage::Finalize.into());

        let nav = Self::nav_row(back, next);

        widget::column![input_seed, nav]
            .width(Length::Shrink)
            .height(Length::Shrink)
            .align_items(iced::Alignment::Center)
            .spacing(50)
    }

    fn notification_field(text: &str) -> widget::Text {
        widget::text(text).size(16).width(250)
    }

    fn text_input_field(placeholder: &str, input: &str) -> widget::TextInput<'a, Message> {
        widget::text_input(placeholder, input)
            .size(16)
            .width(250)
            .line_height(LineHeight::Relative(1.5))
    }

    fn seed_word_field(placeholder: &str, input: &str) -> widget::TextInput<'a, Message> {
        widget::text_input(placeholder, input)
            .size(16)
            .width(100)
            .line_height(LineHeight::Relative(2.))
    }

    fn input_seed(&self) -> Column<'a, Message> {
        let mut seed = widget::column![]
            .width(Length::Shrink)
            .height(Length::Shrink)
            .spacing(20);
        let mut row = widget::row![]
            .width(Length::Shrink)
            .height(Length::Shrink)
            .spacing(20);

        for i in 0..24 {
            if i % 4 == 0 && i != 0 {
                seed = seed.push(row);
                row = widget::row![]
                    .width(Length::Shrink)
                    .height(Length::Shrink)
                    .spacing(20);
            }
            let mut word = "";

            if let Some(s) = self.seed_phrase.reference_word(i) {
                word = s
            }

            let text_field = Self::seed_word_field(&format!("Word {}", i + 1), word)
                .id(Id::new(format!("{i}")))
                .on_input(move |string| {
                    let i = i;
                    let input = vec![string];
                    WalletMessage::UpdateInputSeed((i, input)).into()
                })
                .on_paste(move |mut string| {
                    let i = i;
                    let input = string
                        .split_ascii_whitespace()
                        .map(|s| String::from(s))
                        .collect::<Vec<String>>();
                    string.zeroize();
                    WalletMessage::UpdateInputSeed((i, input)).into()
                });

            row = row.push(text_field);
        }
        seed.push(row)
    }

    fn nav_button(text: &str) -> Button<'a, Message> {
        Button::new(
            widget::text(text)
                .size(16)
                .width(50)
                .horizontal_alignment(iced::alignment::Horizontal::Center)
                .vertical_alignment(iced::alignment::Vertical::Center),
        )
    }

    pub fn nav_row(back: Button<'a, Message>, next: Button<'a, Message>) -> Row<'a, Message> {
        let space = widget::Space::with_width(Length::Fill);
        widget::row![back, space, next]
            .width(Length::Fill)
            .align_items(iced::Alignment::Start)
    }
}
