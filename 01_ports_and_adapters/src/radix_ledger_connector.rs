use std::os::fd::AsFd;

use async_trait::async_trait;

use crate::wallet_data_repository::WalletDataRepo;

#[async_trait]
pub trait RadixLedgerConnector {
    type Account;
    type AccountUpdate;
    type AssetSummary;
    type Resources;

    async fn get_asset_summary_for_account(account_address: Self::Account) -> Self::AssetSummary;

    async fn get_asset_summaries_for_accounts(
        account_addresses: impl IntoIterator<Item = Self::Account>,
    ) -> impl FromIterator<Self::AssetSummary>;

    async fn update_account(
        account: Self::Account,
        repo: impl WalletDataRepo,
        resources: impl AsRef<Self::Resources>,
    ) -> Self::Account;

    async fn update_accounts(
        accounts: impl IntoIterator<Item = Self::Account>,
        resources: impl AsRef<Self::Resources>,
    ) -> impl FromIterator<Self::AccountUpdate>;
}
