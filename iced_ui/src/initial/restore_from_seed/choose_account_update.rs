use super::{RestoreFromSeed, Stage};

impl RestoreFromSeed {
    pub fn update_account_selected(&mut self, chunk_index: usize, account_index: usize) {
        if let Some(chunk) = self.accounts_data.accounts.get_mut(chunk_index) {
            if let Some((_, is_selected, _)) = chunk.get_mut(account_index) {
                *is_selected = !*is_selected
            }
        }
    }

    pub fn goto_page_name_accounts(&mut self) {
        self.accounts_data.selected_accounts = self
            .accounts_data
            .accounts
            .iter()
            .flatten()
            .filter_map(|(account, selected, _)| selected.then_some(account.clone()))
            .collect();

        self.stage = Stage::NameAccounts;
    }
}
