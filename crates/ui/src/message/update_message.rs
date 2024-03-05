use debug_print::debug_println;
use iced::Command;
use iced::futures::SinkExt;

use types::{Action, Update};

use crate::{message::Message, App};


#[derive(Debug, Clone)]
pub struct UpdateMessage(pub(crate)Update);


impl<'a> UpdateMessage {
    pub fn process(self, app: &'a mut App) -> Command<Message> {
        let mut command = Command::none();
        match self.0 {
            Update::Sender(action_tx) => app.action_tx = Some(action_tx),
            Update::Icons(icons) => {
                debug_println!(
                    "{}:{} Received {}icons:",
                    module_path!(),
                    line!(),
                    icons.len()
                );
                app.appview.resource_icons = icons;
            }
            Update::Accounts(accounts) => match app.db {
                Some(ref mut db) => {
                    db.update_accounts(accounts.as_slice())
                        .unwrap_or_else(|err| {
                            debug_println!("Unable to update accounts: {err}");
                        });
                    for account in &accounts {
                        db.update_fungibles_for_account(&account.fungibles, &account.address)
                            .unwrap_or_else(|err| {
                                debug_println!(
                                    "{}:{} Unable to update fungibles: {err}",
                                    module_path!(),
                                    line!()
                                );
                            });

                        if let Some(non_fungibles) = &account.non_fungibles {
                            db.update_non_fungibles_for_account(non_fungibles, &account.address)
                                .unwrap_or_else(|err| {
                                    debug_println!(
                                        "{}:{} Unable to update non fungible: {err}",
                                        module_path!(),
                                        line!()
                                    )
                                });
                        }
                    }
                }
                None => {
                    debug_println!("{}:{}No database found", module_path!(), line!())
                }
            },
            Update::DatabaseLoaded => {
                if let Some(ref channel) = app.action_tx {
                    command = {
                        let mut channel = channel.clone();
                        Command::perform(
                            async move { channel.send(Action::UpdateAll).await },
                            |_| Message::None,
                        )
                    }
                }

                // if let Err(err) = channel.send(Action::UpdateAll) {
                //     self.state = State::Error(AppError::Fatal(Box::new(err)))
                // }
            }
            _ => {}
        }
        command
    }
}
