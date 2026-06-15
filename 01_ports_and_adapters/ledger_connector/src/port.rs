use async_trait::async_trait;

pub type AccountId = types::address::AccountAddress;
pub type Account = types::Account;
pub type Resource = types::Resource;
pub type ResourceId = types::address::ResourceAddress;
pub type AssetSummary = types::AccountSummary;
pub type AccountUpdate = types::collections::AccountUpdate;

pub enum LedgerConnectorError {}

#[async_trait]
pub trait LedgerConnector {
    async fn get_asset_summary_for_account(&self, account_address: AccountId) -> AssetSummary;

    async fn get_asset_summaries_for_accounts(
        &self,
        account_addresses: impl IntoIterator<Item = AccountId> + Send,
    ) -> impl FromIterator<AssetSummary>;

    async fn update_account(
        &self,
        account: Account,
        resources: impl IntoIterator<Item = (ResourceId, Resource)> + Send,
    ) -> AccountUpdate;

    async fn update_accounts(
        &self,
        accounts: impl IntoIterator<Item = Account> + Send,
        resources: impl IntoIterator<Item = (ResourceId, Resource)> + Send,
    ) -> impl FromIterator<AccountUpdate>;
}
