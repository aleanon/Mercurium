use std::collections::{BTreeSet, HashMap};

use debug_print::debug_println;

use iced::widget::image::Handle;
use iced::Task;

use store::{AsyncDb, DbError};
use types::{
    address::ResourceAddress,
    assets::FungibleAsset,
    collections::{AccountsUpdate, AppdataFromDisk},
    AppError, Network,
};

use crate::{
    app::{AppMessage, AppState},
    external_tasks, App,
};

#[derive(Debug, Clone)]
pub enum Message {
    AccountsUpdated(AccountsUpdate),
    AccountsAndResources(AppdataFromDisk),
    Icons((Network, HashMap<ResourceAddress, Handle>)),
    WalletCreated,
    Error(AppError),
}

impl Into<AppMessage> for Message {
    fn into(self) -> AppMessage {
        AppMessage::TaskResponse(self)
    }
}

impl App {
    pub fn process_task_response(&mut self, message: Message) -> Task<AppMessage> {
        match message {
            Message::AccountsUpdated(accounts_update) => {
                return self.process_updated_accounts_and_resources(accounts_update)
            }
            Message::Icons((network, icons)) => {
                if network == self.app_data.settings.network {
                    return self.store_icons_in_app_data(icons);
                }
            }
            Message::AccountsAndResources(accounts_and_resources) => {
                return self.place_accounts_and_resources_in_memory(accounts_and_resources)
            }
            Message::WalletCreated => return self.wallet_created(),
            Message::Error(err) => self.handle_error(err),
        }
        Task::none()
    }

    fn process_updated_accounts_and_resources(
        &mut self,
        accounts_update: AccountsUpdate,
    ) -> Task<AppMessage> {
        if accounts_update.network == self.app_data.settings.network {
            for account_update in &accounts_update.account_updates {
                match self
                    .app_data
                    .fungibles
                    .get_mut(&account_update.account.address)
                {
                    Some(fungibles) => {
                        for (_, asset) in &account_update.fungibles {
                            fungibles.replace(asset.clone());
                        }
                    }
                    None => {
                        let updated_fungibles = account_update
                            .fungibles
                            .iter()
                            .map(|(_, asset)| asset.clone())
                            .collect::<BTreeSet<FungibleAsset>>();
                        self.app_data
                            .fungibles
                            .insert(account_update.account.address.clone(), updated_fungibles);
                    }
                }

                match self
                    .app_data
                    .non_fungibles
                    .get_mut(&account_update.account.address)
                {
                    Some(non_fungibles) => {
                        for (_, asset) in &account_update.non_fungibles {
                            non_fungibles.replace(asset.clone());
                        }
                    }
                    None => {
                        let updated_non_fungibles = account_update
                            .non_fungibles
                            .iter()
                            .map(|(_, asset)| asset.clone())
                            .collect::<BTreeSet<_>>();
                        self.app_data.non_fungibles.insert(
                            account_update.account.address.clone(),
                            updated_non_fungibles,
                        );
                    }
                }

                match self
                    .app_data
                    .accounts
                    .get_mut(&account_update.account.address)
                {
                    Some(account) => {
                        account.balances_last_updated =
                            account_update.account.balances_last_updated;
                    }
                    None => {
                        self.app_data.accounts.insert(
                            account_update.account.address.clone(),
                            account_update.account.clone(),
                        );
                    }
                }
            }
            self.app_data
                .resources
                .extend(accounts_update.new_resources.clone());
        }

        let download_icons = {
            let icon_urls = accounts_update.icon_urls;
            let network = self.app_data.settings.network;
            Task::perform(
                async move {
                    let icons = handles::image::download::download_resize_and_store_resource_icons(
                        icon_urls, network,
                    )
                    .await;
                    (network, icons)
                },
                |(network, icons)| Message::Icons((network, icons)).into(),
            )
        };

        let save_accounts_and_resources_to_disk = {
            let account_updates = accounts_update.account_updates;
            let new_resources = accounts_update
                .new_resources
                .into_iter()
                .map(|(_, resource)| resource)
                .collect::<Vec<_>>();
            let network = self.app_data.settings.network;
            Task::perform(
                async move {
                    let Some(db) = AsyncDb::get(network) else {
                        return Err(DbError::DatabaseNotFound);
                    };
                    db.upsert_resources(new_resources).await?;
                    for account_update in account_updates {
                        debug_println!(
                            "Attempting to save {} fungibles to disk",
                            account_update.fungibles.len()
                        );

                        let fungibles = account_update
                            .fungibles
                            .into_iter()
                            .map(|(_, fungible)| fungible)
                            .collect::<Vec<_>>();
                        db.upsert_fungible_assets_for_account(
                            account_update.account.address.clone(),
                            fungibles,
                        )
                        .await?;

                        debug_println!(
                            "Attempting to save {} non fungibles to disk",
                            account_update.non_fungibles.len()
                        );
                        let non_fungibles = account_update
                            .non_fungibles
                            .into_iter()
                            .map(|(_, non_fungible)| non_fungible)
                            .collect::<Vec<_>>();
                        db.upsert_non_fungible_assets_for_account(
                            account_update.account.address.clone(),
                            &non_fungibles,
                        )
                        .await?;

                        db.upsert_account(account_update.account).await?;
                    }
                    Ok::<_, DbError>(())
                },
                |result| match result {
                    Ok(_) => AppMessage::None,
                    Err(err) => {
                        debug_println!("Failed to save accounts and resources to disk: {}", err);
                        AppMessage::None
                    }
                },
            )
        };

        Task::batch([download_icons, save_accounts_and_resources_to_disk])
    }

    fn store_icons_in_app_data(
        &mut self,
        icons: HashMap<ResourceAddress, Handle>,
    ) -> Task<AppMessage> {
        for (resource_address, icon) in icons {
            self.app_data.resource_icons.insert(resource_address, icon);
        }

        Task::none()
    }

    fn wallet_created(&mut self) -> Task<AppMessage> {
        self.app_state = AppState::Unlocked;

        external_tasks::update_accounts(self.app_data.settings.network)
    }

    fn place_accounts_and_resources_in_memory(
        &mut self,
        accounts_and_resources: AppdataFromDisk,
    ) -> Task<AppMessage> {
        self.app_data.accounts = accounts_and_resources.accounts;
        self.app_data.resources = accounts_and_resources.resources;
        self.app_data.fungibles = accounts_and_resources.fungible_assets;
        self.app_data.non_fungibles = accounts_and_resources.non_fungible_assets;

        Task::none()
    }
}
