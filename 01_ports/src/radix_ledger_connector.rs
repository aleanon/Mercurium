use std::os::fd::AsFd;

use crate::wallet_data_repository::WalletDataRepo;

pub trait RadixLedgerConnector {
    type Account;
    type AccountUpdate;
    type AssetSummary;
    type Resources;

    fn get_asset_summary_for_account(account_address: Self::Account) -> Self::AssetSummary;

    fn get_asset_summaries_for_accounts(
        account_addresses: impl IntoIterator<Item = Self::Account>,
    ) -> impl FromIterator<Self::AssetSummary>;

    fn update_account(
        account: Self::Account,
        repo: impl WalletDataRepo,
        resources: impl AsRef<Self::Resources>,
    ) -> Self::Account;

    fn update_accounts(
        accounts: impl IntoIterator<Item = Self::Account>,
        resources: impl AsRef<Self::Resources>,
    ) -> impl FromIterator<Self::AccountUpdate>;
}
