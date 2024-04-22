use iced::widget::image::Handle;
use std::collections::HashMap;

use super::{Action, EntityAccount, ResourceAddress};

#[derive(Debug, Clone, Default)]
pub enum Update {
    #[default]
    None,
    Sender(iced::futures::channel::mpsc::Sender<Action>),
    Icons(HashMap<ResourceAddress, Handle>),
    DatabaseLoaded,
    Transaction,
    Account(EntityAccount),
    Accounts(Vec<EntityAccount>),
    Error(String),
}
