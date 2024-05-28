use crate::{AccountAddress, Network};

use super::{crypto::Key, Account};

#[derive(Debug, Default)]
pub enum Action {
    #[default]
    None,
    LoadDatabase(Key),
    CheckAndUpdateAll,
    CheckAndUpdateAccounts(Vec<Account>),
    UpdateAccount(AccountAddress),
    SetNetwork(Network),
    UpdateAll,
}


