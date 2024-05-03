use std::collections::BTreeMap;

use handles::EncryptedMnemonic;
use iced::{command, futures::SinkExt, Command};
use types::Action;
use zeroize::Zeroize;

use crate::{
    app::AppData,
    message::{app_view_message::AppViewMessage, Message},
    view::app_view::overlay::{
        add_account_view::{AddAccountView, View},
        Overlay,
    },
    App, CREDENTIALS_STORE_NAME,
};

use super::OverlayMessage;

#[derive(Debug, Clone)]
pub enum AddAccountMessage {
    InputAccountName(String),
    InputPassword(String),
    Next,
    Submit,
}

impl Into<Message> for AddAccountMessage {
    fn into(self) -> Message {
        Message::AppView(AppViewMessage::OverlayMessage(
            OverlayMessage::AddAccountMessage(self),
        ))
    }
}

impl AddAccountMessage {
    pub fn update(self, app: &mut App) -> Command<Message> {
        let mut command = Command::none();

        if let Some(Overlay::AddAccount(add_account_view)) = &mut app.appview.overlay {
            match self {
                Self::InputAccountName(input) => Self::update_account_name(input, add_account_view),
                Self::InputPassword(input) => Self::update_password(input, add_account_view),
                Self::Next => Self::next(add_account_view),
                Self::Submit => command = Self::submit(add_account_view, &mut app.app_data),
            }
        }
        command
    }

    fn update_account_name(input: String, add_account_view: &mut AddAccountView) {
        if !input.is_empty() && !add_account_view.notification.is_empty() {
            add_account_view.notification.clear()
        }
        add_account_view.account_name = input;
    }

    fn update_password(mut input: String, add_account_view: &mut AddAccountView) {
        add_account_view.password.clear();
        add_account_view.password.push_str(input.as_str());
        input.zeroize();
    }

    fn next(add_account_view: &mut AddAccountView) {
        match add_account_view.view {
            View::InputAccountName => {
                if add_account_view.account_name.len() > 0 {
                    add_account_view.notification.clear();
                    add_account_view.view = View::InputPassword
                } else {
                    add_account_view.notification = "Account name cannot be empty".to_string();
                }
            }
            View::InputPassword => {}
        };
    }

    fn submit(add_account_view: &mut AddAccountView, app_data: &mut AppData) -> Command<Message> {
        let mut command = Command::none();
        let account =
            EncryptedMnemonic::from_store(CREDENTIALS_STORE_NAME).and_then(|encrypted_mnemonic| {
                encrypted_mnemonic
                    .decrypt_mnemonic(&add_account_view.password)
                    .and_then(|mnemonic| {
                        let accounts = app_data.db.get_accounts_map().unwrap_or(BTreeMap::new());
                        let mut id = 0;
                        let mut new_account_index = 0;

                        for (_, account) in accounts {
                            if account.id >= id {
                                id = account.id + 1
                            };
                            let account_index = account.derivation_index();
                            if account_index >= new_account_index {
                                new_account_index = account_index + 1
                            };
                        }
                        Ok(handles::wallet::create_account_from_mnemonic(
                            &mnemonic,
                            id,
                            new_account_index,
                            add_account_view.account_name.clone(),
                            app_data.settings.network,
                        ))
                    })
            });
        match account {
            Ok(account) => match app_data.db.upsert_account(&account) {
                Ok(()) => {
                    let mut sender = app_data.backend_sender.clone();
                    command = Command::perform(
                        async move { sender.send(Action::UpdateAccount(account.address)).await },
                        |_| Message::None,
                    )
                }
                Err(err) => {
                    add_account_view.notification =
                        format!("Unable to save account to database: {err}");
                }
            },
            Err(err) => add_account_view.notification = format!("Unable to create account: {err}"),
        };

        command
    }
}
