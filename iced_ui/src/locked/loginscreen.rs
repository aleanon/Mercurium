use std::collections::HashMap;

use debug_print::debug_println;
use iced::{
    theme::Button,
    widget::{self, image::Handle, text::LineHeight, text_input::Id},
    Command, Element, Length,
};
use store::{Db, DbError, IconCache};
use types::{crypto::Password, AccountsAndResources, AppError, ResourceAddress};
use zeroize::Zeroize;

use crate::{app::AppData, app::AppMessage, update::Update};

#[derive(Debug, Clone)]
pub enum Message {
    TextInputChanged(String),
    Login,
}

impl Into<AppMessage> for Message {
    fn into(self) -> AppMessage {
        AppMessage::Login(self)
    }
}

#[derive(Debug, Clone)]
pub struct LoginScreen {
    pub(crate) password: Password,
}

impl<'a> LoginScreen {
    pub fn new() -> Self {
        Self {
            password: Password::new(),
        }
    }

    pub fn password(&self) -> &Password {
        &self.password
    }

    pub fn update(&mut self, message: Message, appdata: &'a mut AppData) -> Command<AppMessage> {
        let mut command = Command::none();

        match message {
            Message::TextInputChanged(mut string) => {
                self.password.clear();
                self.password.push_str(string.as_str());
                string.zeroize()
            }
            Message::Login => {
                // let salt
                let (key, _salt) = self.password.derive_new_db_encryption_key().unwrap();
                //take the password, verify and create encryption key and decrypt the database

                // if let Err(err) = appdata.login() {
                //     match err {
                //         AppError::Fatal(_) => appdata.app_state = AppState::Error(err.to_string()),
                //         AppError::NonFatal(_) => { /*In app notification*/ }
                //     }
                // };

                // if let Err(err) = app.action_tx.send(Action::LoadDatabase(key)) {
                //     app.state = State::Error(AppError::Fatal(Box::new(err)))
                // }

                let login = {
                    Command::perform(async {}, |_| {
                        AppMessage::Common(crate::common::Message::PerformLogin(key))
                    })
                };

                #[cfg(not(feature = "noupdate"))]
                let update_accounts = {
                    let network = appdata.settings.network;
                    let db = Db::load(network).unwrap();
                    Command::perform(
                        async move {
                            handles::radix_dlt::updates::update_all_accounts(network.into(), db)
                                .await
                        },
                        |accounts_update| {
                            #[cfg(debug_assertions)]
                            for account_update in &accounts_update.account_updates {
                                debug_println!("Found {} new fungibles and {} new non_fungibles for account: {}", account_update.fungibles.len(), account_update.non_fungibles.len(), account_update.account.address.as_str());
                            }
                            debug_println!(
                                "Found {} new resources",
                                accounts_update.new_resources.len()
                            );

                            AppMessage::Update(Update::Accounts(accounts_update))
                        },
                    )
                };

                let get_accounts_and_resources = {
                    let network = appdata.settings.network;
                    Command::perform(
                        async move {
                            let db = Db::load(network)?;
                            let accounts = db.get_accounts().unwrap_or_else(|err| {
                                debug_println!("Failed to retrieve accounts: {}", err);
                                HashMap::new()
                            });
                            let resources = db.get_all_resources().unwrap_or_else(|err| {
                                debug_println!("Failed to retrieve resources: {}", err);
                                HashMap::new()
                            });
                            let fungible_assets = db
                                .get_all_fungible_assets_set_per_account()
                                .unwrap_or_else(|err| {
                                    debug_println!("Failed to retrieve fungible assets: {}", err);
                                    HashMap::new()
                                });
                            let non_fungible_assets = db
                                .get_all_non_fungible_assets_set_per_account()
                                .unwrap_or_else(|err| {
                                    debug_println!(
                                        "Failed to retrieve non fungible assets: {}",
                                        err
                                    );
                                    HashMap::new()
                                });

                            Ok::<_, DbError>(AccountsAndResources {
                                accounts,
                                resources,
                                fungible_assets,
                                non_fungible_assets,
                            })
                        },
                        |result| match result {
                            Ok(accounts_and_resources) => {
                                debug_println!(
                                    "Retrieved {} accounts from disk",
                                    accounts_and_resources.accounts.len()
                                );
                                #[cfg(debug_assertions)]
                                for account in &accounts_and_resources.accounts {
                                    if let Some(fungibles) =
                                        accounts_and_resources.fungible_assets.get(&account.0)
                                    {
                                        debug_println!(
                                            "Retrieved {} fungible assets for account: {}",
                                            fungibles.len(),
                                            &account.1.name
                                        );
                                    }
                                    if let Some(non_fungibles) =
                                        accounts_and_resources.non_fungible_assets.get(&account.0)
                                    {
                                        debug_println!(
                                            "Retrieved {} non fungible assets for account: {}",
                                            non_fungibles.len(),
                                            &account.1.name
                                        );
                                    }
                                }
                                debug_println!(
                                    "Retrieved {} resources from disk",
                                    accounts_and_resources.resources.len()
                                );

                                AppMessage::Update(Update::AccountsAndResources(
                                    accounts_and_resources,
                                ))
                            }
                            Err(err) => {
                                debug_println!("Error when opening database: {}", err);
                                AppMessage::None
                            }
                        },
                    )
                };

                let get_resource_icons = {
                    let network = appdata.settings.network;
                    Command::perform(
                        async move {
                            let icon_cache = IconCache::load(network)
                                .await
                                .map_err(|err| AppError::Fatal(Box::new(err)))?;

                            let icons_data = icon_cache
                                .get_all_resource_icons()
                                .await
                                .unwrap_or_else(|err| {
                                    debug_println!("Failed to retrieve resource icons: {}", err);
                                    HashMap::new()
                                });

                            Ok::<_, AppError>(
                                icons_data
                                    .into_iter()
                                    .map(|(resource_address, data)| {
                                        let handle = Handle::from_memory(data);

                                        (resource_address, handle)
                                    })
                                    .collect::<HashMap<ResourceAddress, Handle>>(),
                            )
                        },
                        |result| match result {
                            Ok(icons) => AppMessage::Update(Update::Icons(icons)),
                            Err(err) => {
                                debug_println!("Error when loading icon cache: {}", err);
                                AppMessage::None
                            }
                        },
                    )
                };

                command = Command::batch([
                    #[cfg(not(feature = "noupdate"))]
                    update_accounts,
                    login,
                    get_accounts_and_resources,
                    get_resource_icons,
                ])
            }
        }
        command
    }

    pub fn view(&self) -> Element<'a, AppMessage> {
        let text_field = widget::text_input("Enter Password", &self.password.as_str())
            //.password()
            .secure(true)
            .width(250)
            .line_height(LineHeight::Relative(2.))
            .on_submit(Message::Login.into())
            .size(15)
            .id(Id::new("password_input"))
            .on_input(|value| Message::TextInputChanged(value).into());

        let login_button = widget::Button::new(
            widget::text("Login")
                .size(15)
                .horizontal_alignment(iced::alignment::Horizontal::Center)
                .width(Length::Fill)
                .height(Length::Fill)
                .vertical_alignment(iced::alignment::Vertical::Center),
        )
        .height(30)
        .width(100)
        .style(Button::Primary)
        .on_press(Message::Login.into());

        let col = widget::column![text_field, login_button]
            .height(Length::Shrink)
            .width(Length::Shrink)
            .align_items(iced::Alignment::Center)
            .spacing(30);

        widget::container(col)
            .center_x()
            .center_y()
            .width(Length::Fill)
            .height(Length::Fill)
            .into()
    }
}
