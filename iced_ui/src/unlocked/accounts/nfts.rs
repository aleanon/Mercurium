use types::address::AccountAddress;

use super::non_fungible_view::NonFungibleView;




pub struct NonFungibles {
    account_address: AccountAddress,
    selected: Option<NonFungibleView>
}

impl<'a> NonFungibles {
    pub fn new(account_address: AccountAddress) -> Self {
        Self {
            account_address,
            selected: None
        }
    }
}

