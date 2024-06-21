use debug_print::debug_println;
use iced::{widget::image::Handle, Command};
use store::IconCache;
use types::{assets::FungibleAsset, Fungible, ResourceAddress};

use crate::{
    message::{
        app_view_message::{accounts_message::AccountsViewMessage, AppViewMessage},
        Message,
    },
    view::app_view::{
        accounts_view::{account_view::AssetView, fungible_view::FungibleView, AccountsView},
        ActiveTab,
    },
    App,
};

use super::AccountViewMessage;

#[derive(Debug, Clone)]
pub enum FungiblesMessage {
    Back,
    SelectFungible(FungibleAsset),
    InsertFungibleImage(Vec<u8>),
}

impl Into<Message> for FungiblesMessage {
    fn into(self) -> Message {
        Message::AppView(AppViewMessage::AccountsViewMessage(
            AccountsViewMessage::AccountViewMessage(AccountViewMessage::FungiblesMessage(self)),
        ))
    }
}

impl<'a> FungiblesMessage {
    pub fn process(self, app: &'a mut App) -> Command<Message> {
        match self {
            Self::Back => Self::back(app),
            Self::SelectFungible(fungible) => Self::select_fungible(fungible, app),
            Self::InsertFungibleImage(image_data) => Self::insert_fungible_image(image_data, app),
        }
    }

    fn back(_app: &'a mut App) -> Command<Message> {
        Command::none()
    }

    fn select_fungible(fungible: FungibleAsset, app: &'a mut App) -> Command<Message> {
        if let ActiveTab::Accounts(AccountsView::Account(ref mut account_view)) =
            app.appview.active_tab
        {
            if let AssetView::Fungibles(ref mut fungibles_view) = account_view.view {
                let address = fungible.resource_address.clone();
                fungibles_view.selected = Some(FungibleView::new(fungible, None));

                let network = app.app_data.settings.network;
                Command::perform(
                    async move {
                        let icon_cache = IconCache::load(network).await?;
                        icon_cache.get_resource_icon(address).await
                    },
                    |result| match result {
                        Ok((_, icon_data)) => {
                            FungiblesMessage::InsertFungibleImage(icon_data).into()
                        }
                        Err(_) => {
                            debug_println!("Could not find image");
                            Message::None
                        }
                    },
                )
            } else {
                unreachable!("{}:{} Wrong State", module_path!(), line!())
            }
        } else {
            unreachable!("{}:{} Wrong State", module_path!(), line!())
        }
    }

    fn insert_fungible_image(image_data: Vec<u8>, app: &'a mut App) -> Command<Message> {
        if let ActiveTab::Accounts(AccountsView::Account(ref mut account_view)) =
            app.appview.active_tab
        {
            if let AssetView::Fungibles(ref mut fungibles_view) = account_view.view {
                if let Some(ref mut fungible_view) = fungibles_view.selected {
                    fungible_view.image = Some(Handle::from_memory(image_data))
                }
            }
        }
        Command::none()
    }
}
