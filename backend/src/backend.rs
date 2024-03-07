

use std::{collections::HashMap, num::NonZeroU32, path::PathBuf};

use anyhow::Result;
use bytes::Bytes;
use debug_print::debug_println;
use iced::widget::image::Handle as ImageHandle;
use iced::{
    futures::channel::mpsc::Receiver as MpscReceiver,
    futures::{channel::mpsc::Sender, SinkExt},
    Subscription,
};
use std::sync::mpsc::{self as Mpsc, Sender as MpscSender};

use handles::filesystem::{app_path::AppPath, resize_image::resize_image};
use types::{action::Action, update::Update, ResourceAddress};

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

        //self.actions().await?;

        Ok(())
    }

    // pub async fn actions(&mut self) -> Result<()> {
    //     debug_println!("{}:{} Listening for Actions", module_path!(), line!());

    //     while let Ok(action) = self.from_frontend_rx {
    //         debug_println!(
    //             "{}:{} Backend Action received: {:?}",
    //             module_path!(),
    //             line!(),
    //             &action
    //         );

    //         match action {
    //             Action::LoadDatabase(key) => {
    //                 if let None = self.handle.db {
    //                     match self.load_database() {
    //                         Ok(()) => {
    //                             if let Err(err) = self.to_frontend_tx.send(Update::DatabaseLoaded) {
    //                                 debug_println!(
    //                                     "{}:{} Failed to send update to frontend: {err}",
    //                                     module_path!(),
    //                                     line!()
    //                                 )
    //                             }
    //                         }
    //                         Err(err) => self.to_frontend_tx.send(Update::Error(format!(
    //                             "Unable to load database: {:?}",
    //                             err
    //                         )))?,
    //                     }
    //                 } else {
    //                     if let Err(err) = self.to_frontend_tx.send(Update::DatabaseLoaded) {
    //                         debug_println!(
    //                             "{}:{} Failed to send update to frontend: {err}",
    //                             module_path!(),
    //                             line!()
    //                         )
    //                     }
    //                 }
    //             }
    //             Action::UpdateAll => match self.handle.update_accounts().await {
    //                 Ok(accounts) => {
    //                     debug_println!(
    //                         "{}:{} Update successfull {} account(s) updated",
    //                         module_path!(),
    //                         line!(),
    //                         accounts.len()
    //                     );

    //                     self.to_frontend_tx.send(Update::Accounts(accounts))?;
    //                 }
    //                 Err(err) => {
    //                     debug_println!(
    //                         "{}:{} Unable to update accounts: {err}",
    //                         module_path!(),
    //                         line!()
    //                     );

    //                     self.to_frontend_tx.send(Update::Error(format!(
    //                         "Unable to update accounts: {:?}",
    //                         err
    //                     )))?;
    //                 }
    //             },
    //             Action::CheckAndUpdateAccounts(accounts) => {
    //                 match self.handle.update_accounts().await {
    //                     Ok(accounts) => {
    //                         if let Err(err) = self.to_frontend_tx.send(Update::Accounts(accounts)) {
    //                             debug_println!(
    //                                 "{}:{}Failed to send update to frontend {err}",
    //                                 module_path!(),
    //                                 line!()
    //                             )
    //                         }
    //                     }
    //                     Err(err) => {
    //                         if let Err(err) = self.to_frontend_tx.send(Update::Error(format!(
    //                             "Unable to update accounts: {:?}",
    //                             err
    //                         ))) {
    //                             debug_println!(
    //                                 "{}:{} Failed to send update to frontend: {err}",
    //                                 module_path!(),
    //                                 line!()
    //                             )
    //                         }
    //                     }
    //                 }
    //             }
    //             _ => {}
    //         }
    //     }
    //     Ok(())
    // }

    pub fn backend_subscription() -> Subscription<Update> {
        struct BackEndWorker;

        iced::subscription::channel(
            std::any::TypeId::of::<BackEndWorker>(),
            50,
            |mut output| async move {
                //let dummy_channel = Mpsc::channel();
                let (action_tx, action_rx) = iced::futures::channel::mpsc::channel(50);
                let mut backend = BackEnd::new(action_rx).unwrap();
                backend.load(&mut output).await.unwrap();

                if let Err(err) = output.send(Update::Sender(action_tx)).await {
                    debug_println!(
                        "{}:{} Unable to send transmitter {err}",
                        module_path!(),
                        line!()
                    )
                }

                loop {
                    use iced::futures::StreamExt;
                    let action = backend.action_rx.select_next_some().await;

                    match action {
                        Action::LoadDatabase(key) => backend.action_load_db(&mut output).await,
                        Action::UpdateAll => backend.action_update_all(&mut output).await,
                        _ => {
                            output.send(Update::None).await;
                        }
                    }
                }
            },
        )
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
                    NonZeroU32::new(50).unwrap(),
                    NonZeroU32::new(50).unwrap(),
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
