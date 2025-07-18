use crate::services::wallet_repository::Repository;

pub trait AccountRepository: Repository {
    type Account;
    type AccountId;

    fn upsert_account(&mut self, account: Self::Account) -> Result<(), Self::Error>;

    fn upsert_accounts<Accounts: IntoIterator<Item = Self::Account>>(
        &mut self,
        accounts: Accounts,
    ) -> Result<(), Self::Error>;

    fn get_account(&self, account_id: Self::AccountId) -> Result<Self::Account, Self::Error>;

    fn get_all_accounts<Accounts: FromIterator<Self::Account>>(
        &self,
    ) -> Result<Accounts, Self::Error>;

    fn delete_account(&self, account_id: Self::AccountId) -> Result<(), Self::Error>;
}
