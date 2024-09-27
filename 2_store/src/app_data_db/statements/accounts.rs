pub const CREATE_TABLE_ACCOUNTS: &'static str = "CREATE TABLE IF NOT EXISTS 
    accounts (
        address BLOB NOT NULL PRIMARY KEY,
        id INTEGER NOT NULL,
        name TEXT NOT NULL,
        network INTEGER NOT NULL,
        derivation_path BLOB NOT NULL,
        public_key BLOB NOT NULL,
        hidden BOOL NOT NULL,
        settings BLOB NOT NULL,
        balances_last_updated INTEGER,
        transactions_last_updated INTEGER
    )
";

pub const UPSERT_ACCOUNT: &'static str = "INSERT INTO 
    accounts (
        address,
        id,
        name,
        network,
        derivation_path,
        public_key,
        hidden,
        settings,
        balances_last_updated,
        transactions_last_updated
    )
    VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
    ON CONFLICT (address)
    DO UPDATE SET 
        id = excluded.id,
        name = excluded.name,
        network = excluded.network,
        derivation_path = excluded.derivation_path,
        public_key = excluded.public_key,
        hidden = excluded.hidden,
        settings = excluded.settings,
        balances_last_updated = excluded.balances_last_updated,
        transactions_last_updated = excluded.transactions_last_updated
";
