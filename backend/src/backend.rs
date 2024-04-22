use std::{collections::HashMap, num::NonZeroU32, path::PathBuf};

use anyhow::Result;
use bytes::Bytes;
use debug_print::debug_println;
use iced::widget::image::Handle as ImageHandle;
use iced::{
    futures::channel::mpsc::Receiver as MpscReceiver,
    futures::{channel::mpsc::Sender, SinkExt},
};

use handles::filesystem::resize_image::resize_image;
use types::{AccountAddress, Action, AppPath, ResourceAddress, Update};

use super::handle::Handle;

use store::AsyncDb;

#[derive(Debug)]
pub struct BackEnd {
    pub(crate) handle: Handle,
    app_path: AppPath,
    //to_frontend_tx: MpscSender<Update>,
    pub action_rx: MpscReceiver<Action>,
}

impl BackEnd {
    pub fn new(
        //to_frontend_tx: MpscSender<Update>,
        from_frontend_rx: MpscReceiver<Action>,
    ) -> Result<Self> {
        debug_println!("{}:{} Initializing backend...", module_path!(), line!());

        let handle = Handle::new()?;

        let app_path = AppPath::new()?.create_directories_if_not_exists()?;

        debug_println!("{}:{} Backend initialized", module_path!(), line!());

        Ok(Self {
            handle,
            app_path,
            //to_frontend_tx,
            action_rx: from_frontend_rx,
        })
    }

    pub async fn load_database(&mut self) -> Result<()> {
        let db = AsyncDb::load().await?;
        self.handle.db = Some(db);
        Ok(())
    }

    pub async fn load(&mut self, output: &mut Sender<Update>) -> Result<()> {
        debug_println!("{}:{} Loading Icons", module_path!(), line!());

        if let Some(icons) = Self::load_icons(&self.app_path.icons_directory()) {
            if let Err(err) = output.send(Update::Icons(icons)).await {
                debug_println!(
                    "{}:{} Failed to send icons to frontend: {err}",
                    module_path!(),
                    line!()
                )
            }
        } else {
            debug_println!("{}:{} No icons found", module_path!(), line!());
        }

        Ok(())
    }

    pub async fn action_load_db(&mut self, output: &mut Sender<Update>) {
        if self.handle.db.is_none() {
            match self.load_database().await {
                Ok(()) => {
                    output
                        .send(Update::DatabaseLoaded)
                        .await
                        .unwrap_or_else(|err| {
                            debug_println!(
                                "{}:{} Failed to send update: {err}",
                                module_path!(),
                                line!()
                            )
                        });
                }
                Err(err) => {
                    output
                        .send(Update::Error(format!("Unable to load database: {:?}", err)))
                        .await
                        .unwrap_or_else(|err| {
                            debug_println!(
                                "{}:{} Failed to send update: Error {err}",
                                module_path!(),
                                line!()
                            )
                        });
                }
            }
        } else {
            output
                .send(Update::DatabaseLoaded)
                .await
                .unwrap_or_else(|err| {
                    debug_println!(
                        "{}:{} Failed to send update: DatabaseLoaded {err}",
                        module_path!(),
                        line!()
                    )
                });
        }
    }

    pub async fn action_update_all(&mut self, output: &mut Sender<Update>) {
        match self.handle.update_accounts().await {
            Ok(accounts) => {
                debug_println!(
                    "{}:{} Update successfull {} account(s) updated",
                    module_path!(),
                    line!(),
                    accounts.len()
                );

                output
                    .send(Update::Accounts(accounts))
                    .await
                    .unwrap_or_else(|err| {
                        debug_println!(
                            "{}:{} Failed to send Update: {err}",
                            module_path!(),
                            line!()
                        )
                    });
            }
            Err(err) => {
                debug_println!(
                    "{}:{} Unable to update accounts: {err}",
                    module_path!(),
                    line!()
                );

                output
                    .send(Update::Error(format!(
                        "Unable to update accounts: {:?}",
                        err
                    )))
                    .await
                    .unwrap_or_else(|err| {
                        debug_println!(
                            "{}:{} Failed to send Update: {err}",
                            module_path!(),
                            line!()
                        )
                    });
            }
        }
    }

    pub async fn action_update_account(
        &mut self,
        account_address: AccountAddress,
        output: &mut Sender<Update>,
    ) {
        let updated_account = self.handle.update_account(account_address).await;

        if let Ok(entity_account) = updated_account {
            output.send(Update::Account(entity_account)).await.ok();
        }
    }

    pub fn load_icons(directory: &PathBuf) -> Option<HashMap<ResourceAddress, ImageHandle>> {
        let icons = std::fs::read_dir(directory)
            .ok()?
            .into_iter()
            .filter_map(|result| {
                let file = result
                    .inspect_err(|err| {
                        debug_println!("{}:{} Could not get file {err}", module_path!(), line!())
                    })
                    .ok()?;
                let file_path = file.path();
                let file_name = file_path.file_stem()?.to_os_string();
                let address = ResourceAddress::try_from(&file_name)
                    .inspect_err(|err| {
                        debug_println!(
                            "{}:{} Could not convert to resource address {err}",
                            module_path!(),
                            line!()
                        )
                    })
                    .ok()?;
                let image = image::open(file_path).ok()?;
                let resized = resize_image(
                    &image,
                    NonZeroU32::new(40).unwrap(),
                    NonZeroU32::new(40).unwrap(),
                )?;
                Some((
                    address,
                    ImageHandle::from_memory(Bytes::from(resized.buffer().to_vec())),
                ))
            })
            .collect::<HashMap<ResourceAddress, ImageHandle>>();

        if icons.is_empty() {
            debug_println!("{}:{} No icons found", module_path!(), line!());
            None
        } else {
            Some(icons)
        }
    }
}
