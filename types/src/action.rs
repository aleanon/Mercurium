use super::{crypto::Key, Account};

#[derive(Debug, Default)]
pub enum Action {
    #[default]
    None,
    LoadDatabase(Key),
    CheckAndUpdateAll,
    CheckAndUpdateAccounts(Vec<Account>),
    UpdateAll,
}
