use std::{
    collections::{BTreeSet, HashMap},
    io::{BufWriter, Cursor},
    num::NonZeroU32,
    path::PathBuf,
    str::FromStr,
    sync::Arc,
};

use image::{imageops::FilterType, ImageFormat};
use store::{DbError, AsyncDb};
use handles::filesystem::{resize_image::resize_image, save_image::save_image};
use types::{
       RadixDecimal, AppPath, ParseAddrError ,Fungible, Fungibles, Icon, MetaData, NFIDs, NonFungible, NonFungibles, ResourceAddress, EntityAccount, response_models::{NonFungibleResource, FungibleResource, Entity}
    };

use bytes::Bytes;
use debug_print::debug_println;
use iced::futures::{future::{join, join_all}, task::LocalSpawnExt};
use thiserror::Error;
use tokio::task::JoinError;

use crate::coms::{radixrequest::RadixDltRequestError, Coms, ComsError};


#[derive(Debug, Error)]
pub enum HandleError {
    #[error("Unable to create database, source: {0}")]
    FailedToCreateDb(#[from] DbError),
    #[error("Unable to create appdata, source: {0}")]
    ComsError(#[from] ComsError),
    #[error("Request failed, source: {0}")]
    RadixRequestError(#[from] RadixDltRequestError),
    #[error("No details found")]
    NoDetailsFound,
    #[error("Entity not found: {0}")]
    EntityNotFound(String),
    #[error("Invalid account id: {0}")]
    InvalidAccountId(usize),
    #[error("Image Error: {0}")]
    ImageError(#[from] image::ImageError),
    #[error("Unable to join tasks {0}")]
    JoinError(#[from] JoinError),
    #[error("{0}")]
    AddrError(#[from] ParseAddrError),
    #[error("No database")]
    NoDatabase,
}

#[derive(Debug)]
pub struct Handle {
    //pub appdata: AppData,
    pub coms: Arc<Coms>,
    pub db: Option<AsyncDb>,
}

impl Handle {
    pub fn new() -> Result<Self, HandleError> {
        let coms = Arc::new(Coms::new()?);
        Ok(Self { coms, db: None })
    }

    pub async fn update_accounts(&self) -> Result<Vec<EntityAccount>, HandleError> {
        let mut accounts = self
            .db
            .as_ref()
            .ok_or(HandleError::NoDatabase)?
            .get_entityaccounts()
            .await
            .unwrap_or_else(|_| Vec::with_capacity(0));

        let addresses = accounts
            .iter()
            .map(|account| account.address.as_str())
            .collect::<Vec<&str>>();

        // Send a request to the Radix gateway for the details of these accounts
        let accounts_response = self
            .coms
            .radixdlt_request_builder
            .get_entity_details(addresses.as_slice())
            .await?
            .items;

        // Create a task for each account that will get the details of each asset in the account
        let tasks = accounts_response.into_iter().map(|entity_account| {
            
            let coms = self.coms.clone();
            let account = accounts
                .iter()
                .enumerate()
                .find(|(_, account)| account.address.as_str() == entity_account.address.as_str());

            let account = match account {
                Some((i, _)) => Some(accounts.remove(i)),
                None => None,
            };

            tokio::spawn(async move {
                match account {
                    Some(account) => {
                        Self::update_account_data(coms, entity_account, account).await
                    }
                    None => Err(HandleError::EntityNotFound(entity_account.address)),
                }
            })
        });

        let accounts = join_all(tasks)
            .await
            .into_iter()
            .filter_map(|result| {
                #[cfg(debug_assertions)]
                match &result {
                    Ok(value) => match value {
                        Ok(account) => println!(
                            "Successfully retrieved data for account {}",
                            account.address.as_str()
                        ),
                        Err(err) => println!("Failed to retrieve account data, Error: {err}"),
                    },
                    Err(err) => println!("Failed to retrieve account data, Error: {err}"),
                }

                result.ok().and_then(|result| result.ok())
            })
            .collect();

        Ok(accounts)
    }

    pub async fn update_account_data(
        coms: Arc<Coms>,
        account_response: Entity,
        mut account: EntityAccount,
    ) -> Result<EntityAccount, HandleError> {
        let fungible_resources = account_response
            .fungible_resources
            .items
            .into_iter()
            .map(|fungible| (fungible.resource_address.to_owned(), fungible))
            .collect::<HashMap<String, FungibleResource>>();
        let fungible_resources = Arc::new(fungible_resources);

        let non_fungible_resources = account_response
            .non_fungible_resources
            .items
            .into_iter()
            .map(|non_fungible| (non_fungible.resource_address.to_owned(), non_fungible))
            .collect::<HashMap<String, NonFungibleResource>>();
        let non_fungible_resources = Arc::new(non_fungible_resources);

        let coms_clone = coms.clone();

        let fungibles_response =
            tokio::spawn(
                async move { Self::fungibles_response(coms_clone, fungible_resources).await },
            );

        let non_fungibles_response = tokio::spawn(async move {
            Self::non_fungibles_response(coms, non_fungible_resources).await
        });

        let (fungibles, non_fungibles) = join(fungibles_response, non_fungibles_response).await;
        let fungibles = fungibles??;
        let non_fungibles = non_fungibles??;

        account.fungibles = fungibles;
        account.non_fungibles = Some(non_fungibles);
        Ok::<_, HandleError>(account)
    }

    async fn fungibles_response(
        coms: Arc<Coms>,
        fungible_resources: Arc<HashMap<String, FungibleResource>>,
    ) -> Result<Fungibles, HandleError> {
        let fungible_addresses = fungible_resources
            .keys()
            .map(|key| key.as_str())
            .collect::<Vec<_>>();
        let fungibles_details = coms
            .radixdlt_request_builder
            .get_entity_details(fungible_addresses.as_slice())
            .await?;

        let fungible_tasks = fungibles_details.items.into_iter().map(|fungible| {
            let fungible_resources = fungible_resources.clone();
            tokio::spawn(async move { Self::fungible_response(fungible_resources, fungible).await })
        });

        let joined = join_all(fungible_tasks)
            .await
            .into_iter()
            .filter_map(|result| result.ok())
            .collect::<Vec<_>>();

        let fungibles: Fungibles = joined
            .into_iter()
            .filter_map(|result| result.ok())
            .collect::<BTreeSet<_>>()
            .into();

        Ok::<_, HandleError>(fungibles)
    }

    async fn fungible_response(
        fungible_resources: Arc<HashMap<String, FungibleResource>>,
        fungible: Entity,
    ) -> Result<Fungible, HandleError> {
        let (last_updated, amount) = match fungible_resources.get(&*fungible.address) {
            Some(fungible_resource) => {
                let mut amount = RadixDecimal::ZERO;
                let mut last_updated = 0;
                for vault in &fungible_resource.vaults.items {
                    amount += RadixDecimal::from_str(&vault.amount)
                        .unwrap_or_else(|_| RadixDecimal::ZERO);
                    if last_updated < vault.last_updated_at_state_version {
                        last_updated = vault.last_updated_at_state_version
                    }
                }
                (last_updated, amount.into())
            }
            None => (0, RadixDecimal::ZERO.into()),
        };

        let address = ResourceAddress::from_str(&fungible.address)?;

        let mut name = None;
        let mut symbol = None;
        let mut description = None;
        let mut icon_url = None;
        let mut metadata = MetaData::new();
        let current_supply = "fungible.details.total_supply".to_owned();

        for item in fungible.metadata.items {
            match &*item.key {
                "name" => name = item.value.typed.value,
                "symbol" => symbol = item.value.typed.value,
                "description" => description = item.value.typed.value,
                "icon_url" => icon_url = item.value.typed.value.filter(|value| value.len() != 0),
                _ => metadata.push(item.into()),
            }
        }

        let icon = Self::get_icon(icon_url, &address).await;

        let fungible = Fungible {
            address,
            amount,
            current_supply,
            description,
            name: name.unwrap_or(String::with_capacity(0)),
            symbol: symbol.unwrap_or(String::with_capacity(0)),
            icon,
            last_updated_at_state_version: last_updated as i64,
            metadata,
        };
        Ok::<_, HandleError>(fungible)
    }

    async fn non_fungibles_response(
        coms: Arc<Coms>,
        non_fungible_resources: Arc<HashMap<String, NonFungibleResource>>,
    ) -> Result<NonFungibles, HandleError> {
        let non_fungible_addresses = non_fungible_resources
            .keys()
            .map(|key| key.as_str())
            .collect::<Vec<&str>>();

        let non_fungibles_details = coms
            .radixdlt_request_builder
            .get_entity_details(non_fungible_addresses.as_slice())
            .await?;

        let tasks = non_fungibles_details.items.into_iter().map(|non_fungible| {
            let non_fungible_resources = non_fungible_resources.clone();
            tokio::spawn(async move {
                Self::non_fungible_response(non_fungible_resources, non_fungible).await
            })
        });

        let non_fungibles: NonFungibles = join_all(tasks)
            .await
            .into_iter()
            .filter_map(|result| result.ok().and_then(|result| result.ok()))
            .collect::<Vec<_>>()
            .into();

        Ok::<_, HandleError>(non_fungibles)
    }

    async fn non_fungible_response(
        non_fungible_resources: Arc<HashMap<String, NonFungibleResource>>,
        non_fungible: Entity,
    ) -> Result<NonFungible, HandleError> {
        let (last_updated, nfids) = match non_fungible_resources.get(&*non_fungible.address) {
            Some(non_fungible_resource) => {
                let mut last_updated = 0;
                for vault in &non_fungible_resource.vaults.items {
                    if last_updated < vault.last_updated_at_state_version {
                        last_updated = vault.last_updated_at_state_version
                    }
                }
                let nfids = NFIDs::from(&non_fungible_resource.vaults);
                (last_updated, nfids)
            }
            None => (0, NFIDs::new()),
        };

        let address = ResourceAddress::from_str(&non_fungible.address)?;

        let mut name = None;
        let mut symbol = None;
        let mut description = None;
        let mut icon_url = None;
        let mut metadata = MetaData::new();
        let _current_supply = "non_fungible.details.total_supply".to_owned();

        for item in non_fungible.metadata.items {
            match &*item.key {
                "name" => name = item.value.typed.value,
                "symbol" => symbol = item.value.typed.value,
                "description" => description = item.value.typed.value,
                "icon_url" => icon_url = item.value.typed.value.filter(|value| value.len() != 0),
                _ => metadata.push(item.into()),
            }
        }

        let icon = Self::get_icon(icon_url, &address).await;

        let non_fungible = NonFungible {
            address,
            description,
            name: name.unwrap_or(String::with_capacity(0)),
            symbol: symbol.unwrap_or(String::with_capacity(0)),
            icon,
            nfids,
            last_updated_at_state_version: last_updated as i64,
            metadata,
        };
        Ok::<_, HandleError>(non_fungible)
    }

    async fn get_icon(
        icon_url: Option<String>,
        resource_address: &ResourceAddress,
    ) -> Option<Icon> {
        match icon_url {
            Some(ref url) => {
                if let Ok(app_path) = AppPath::new() {
                    let mut icon_path = app_path.icons_directory().clone();
                    icon_path.push(resource_address.as_str());
                    if icon_path.exists() {
                        if let Ok(image) = image::open(&icon_path) {
                            if let Some(resized) = resize_image(
                                &image,
                                NonZeroU32::new(50).unwrap(),
                                NonZeroU32::new(50).unwrap(),
                            ) {
                                Some(Icon::new(Bytes::from(resized.buffer().to_vec())))
                            } else
                            //Could not resize image
                            {
                                Self::download_icon(url, Some(&mut icon_path)).await
                            }
                        } else
                        //Could not open image
                        {
                            Self::download_icon(url, Some(&mut icon_path)).await
                        }
                    } else
                    //Icon path does not exist
                    {
                        Self::download_icon(url, Some(&mut icon_path)).await
                    }
                } else
                //Unable to determine icons directory
                {
                    Self::download_icon(url, None).await
                }
            }
            None => None,
        }
    }

    async fn download_icon(url: &String, icon_path: Option<&mut PathBuf>) -> Option<Icon> {
        let response = reqwest::get(url).await.ok()?;

        let bytes = response.bytes().await.ok()?;
        let reader = image::io::Reader::new(Cursor::new(&bytes));
        let with_guessed_format = reader.with_guessed_format().ok()?;
        let format = with_guessed_format.format()?;
        let image = with_guessed_format.decode().ok()?;
        

        if let Some(path) = icon_path {
            path.set_extension(handles::filesystem::image_extension::get_extension(&format));
            image.save_with_format(path, format).unwrap_or(());
        }

        let resized = image.resize(50, 50, FilterType::Lanczos3);
        let mut write_buffer = BufWriter::new(Cursor::new(Vec::new()));        
        resized.write_to(&mut write_buffer, format).ok()?;

        let inner = write_buffer.into_inner().ok()?.into_inner();
        let icon = Icon::new(Bytes::from(inner));

        Some(icon)         

        // if let Some(response) = response {
        //     let bytes = response.bytes().await.ok();

        //     if let Some(bytes) = bytes {
        //         let reader = image::io::Reader::new(Cursor::new(&bytes));

        //         if let Ok(new_reader) = reader.with_guessed_format() {

        //             if let Ok(image) = new_reader.decode() {

        //                 if let Some(path) = icon_path {
        //                     save_image(&image, path)
        //                 }
        //                 if let Some(write_buffer) = resize_image(
        //                     &image,
        //                     NonZeroU32::new(50).unwrap(),
        //                     NonZeroU32::new(50).unwrap(),
        //                 ) {
        //                     return Some(Icon::new(Bytes::from(write_buffer.buffer().to_vec())));
        //                 } else {
        //                     debug_println!(
        //                         "{}:{} Unable to resize image: {url}",
        //                         module_path!(),
        //                         line!()
        //                     )
        //                 }
        //             } else {
        //                 debug_println!(
        //                     "{}:{} Unable to decode image: {url}",
        //                     module_path!(),
        //                     line!()
        //                 )
        //             }
        //         } else {
        //             debug_println!(
        //                 "{}:{} Unable to guess image format: {url}",
        //                 module_path!(),
        //                 line!()
        //             )
        //         }
        //     } else {
        //         debug_println!(
        //             "{}:{} Unable to get icon bytes: {url}",
        //             module_path!(),
        //             line!()
        //         )
        //     }
        // } else {
        //     debug_println!(
        //         "{}:{} Unable to get image from url: {url}",
        //         module_path!(),
        //         line!()
        //     )
        // }

        // None
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::backend::BackEnd;
    use types::{Account, AccountAddress, Network};
    use scrypto::crypto::Ed25519PublicKey;

    #[tokio::test]
    async fn test_get_account_data() {
        // let (sender, _) = std::sync::mpsc::channel();
        let (_, receiver) = iced::futures::channel::mpsc::channel(50);
        let connection = tokio_rusqlite::Connection::open_in_memory().await.unwrap();
        let mut db = AsyncDb::with_connection(connection).await;
        db.create_table_accounts().await.unwrap();
        db.create_table_fungibles().await.unwrap();
        db.create_table_non_fungibles().await.unwrap();

        let account = Account::new(
            0,
            "some account".to_owned(),
            Network::Mainnet,
            [0u32; 6],
            AccountAddress::from_str(
                "account_rdx12ymqrlezhreuknut5x5ucq30he638pqu9wum7nuxl65z9pjdt2a5ax",
            )
            .unwrap(),
            Ed25519PublicKey([0u8; Ed25519PublicKey::LENGTH]),
        );

        db.update_account(account).await.unwrap();

        let mut backend = BackEnd::new(receiver).unwrap();
        backend.handle.db = Some(db);

        let mut accounts = backend.handle.update_accounts().await.unwrap();
        let account = accounts.remove(0);

        let fungibles = account.fungibles;
        let resource_address = ResourceAddress::from_str(
            "resource_rdx1thlnv2lydu7np9w8guguqslkydv000d7ydn7uq0sestql96hrfml0v",
        )
        .unwrap();

        let found = fungibles
            .iter()
            .find(|fungible| fungible.address == resource_address);
        assert!(found.is_some());
    }
}
