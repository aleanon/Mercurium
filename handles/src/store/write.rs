use debug_print::debug_println;
use store::Db;
use types::{AccountAddress, EntityAccount, Fungibles, NonFungibles};


pub fn save_accounts_to_database(db: &mut Db, accounts: &Vec<EntityAccount>) {
  db.update_accounts(accounts.as_slice())
    .unwrap_or_else(|err| {
        debug_println!("Unable to update accounts: {err}");
    });

  for account in accounts {
    save_fungibles_to_database(db, &account.fungibles, &account.address);

    save_non_fungibles_to_database(db, &account.non_fungibles, &account.address);
  }
}

pub fn save_fungibles_to_database(db:&mut Db, fungibles: &Fungibles, account_address: &AccountAddress) {
  db.update_fungibles_for_account(fungibles, &account_address)
    .unwrap_or_else(|err| {
        debug_println!(
            "{}:{} Unable to update fungibles: {err}",
            module_path!(),
            line!()
        );
    }
  );
}

pub fn save_non_fungibles_to_database(db:&mut Db, non_fungibles: &NonFungibles, account_address: &AccountAddress) {
  db.update_non_fungibles_for_account(non_fungibles, &account_address)
    .unwrap_or_else(|err| {
        debug_println!(
            "{}:{} Unable to update fungibles: {err}",
            module_path!(),
            line!()
        );
    }
  );
}