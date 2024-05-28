pub mod create {
    pub const CREATE_TABLE_ACCOUNTS: &'static str = "CREATE TABLE IF NOT EXISTS 
        accounts (
            address BLOB NOT NULL PRIMARY KEY,
            id INTEGER NOT NULL,
            name TEXT NOT NULL,
            network INTEGER NOT NULL,
            derivation_path BLOB NOT NULL,
            public_key BLOB NOT NULL,
            hidden BOOL NOT NULL,
            settings BLOB NOT NULL
        )
    ";

    pub const CREATE_TABLE_RESOURCES: &'static str = "CREATE TABLE IF NOT EXISTS 
        resources (
            address BLOB NOT NULL PRIMARY KEY,
            name TEXT NOT NULL,
            symbol TEXT NOT NULL,
            total_supply TEXT NOT NULL,
            description TEXT NOT NULL,
            last_updated INTEGER NOT NULL,
            metadata BLOB NOT NULL
        )
    ";

    pub const CREATE_TABLE_FUNGIBLE_ASSETS: &'static str = "CREATE TABLE IF NOT EXISTS 
        fungible_assets (
            id TEXT NOT NULL PRIMARY KEY,
            resource_address BLOB NOT NULL,
            amount TEXT NOT NULL,
            last_updated INTEGER NOT NULL,
            account_address BLOB NOT NULL,
            FOREIGN KEY(resource_address) REFERENCES resources(address),
            FOREIGN KEY(account_address) REFERENCES accounts(address)
        )
    ";

    pub const CREATE_TABLE_NON_FUNGIBLE_ASSETS: &'static str = "CREATE TABLE IF NOT EXISTS 
        non_fungible_assets (
            id TEXT NOT NULL PRIMARY KEY,
            resource_address BLOB NOT NULL,
            nfids BLOB NOT NULL,
            last_updated INTEGER NOT NULL,
            account_address BLOB NOT NULL,
            FOREIGN KEY(resource_address) REFERENCES resources(address),
            FOREIGN KEY(account_address) REFERENCES accounts(address)
        )
    ";

    pub const CREATE_TABLE_TRANSACTIONS: &'static str = "CREATE TABLE IF NOT EXISTS 
        transactions (
            id BLOB NOT NULL PRIMARY KEY,
            timestamp BLOB NOT NULL,
            state_version INTEGER NOT NULL,
            status INTEGER NOT NULL
        )
    ";

    pub const CREATE_TABLE_BALANCE_CHANGES: &'static str = "CREATE TABLE IF NOT EXISTS
        balance_changes (
            id BLOB NOT NULL PRIMARY KEY,
            account BLOB NOT NULL,
            resource BLOB NOT NULL,
            amount TEXT NOT NULL,
            tx_id BLOB NOT NULL,
            FOREIGN KEY(tx_id) REFERENCES transactions(id)
        )
    ";
}

pub mod upsert {
    pub const UPSERT_ACCOUNT: &'static str = "INSERT INTO 
        accounts (
            address,
            id,
            name,
            network,
            derivation_path,
            public_key,
            hidden,
            settings
        )
        VALUES (?, ?, ?, ?, ?, ?, ?, ?)
        ON CONFLICT (address)
        DO UPDATE SET 
            id = excluded.id,
            name = excluded.name,
            network = excluded.network,
            derivation_path = excluded.derivation_path,
            public_key = excluded.public_key,
            hidden = excluded.hidden,
            settings = excluded.settings
        ";

    pub const UPSERT_RESOURCE: &'static str = "INSERT INTO
        resources (
            address,
            name,
            symbol,
            current_supply,
            description,
            last_updated,
            metadata
        )
        VALUES (?, ?, ?, ?, ?, ?, ?)
        ON CONFLICT (address)
        DO UPDATE SET 
            name = excluded.name,
            symbol = excluded.symbol,
            current_supply = excluded.current_supply,
            description = excluded.description,
            public_key = excluded.public_key,
            hidden = excluded.hidden,
            settings = excluded.settings
    ";

    pub const UPSERT_FUNGIBLE_ASSET: &'static str = "INSERT INTO
        fungible_assets (
            id,
            resource_address,
            amount,
            last_updated,
            account_address
        )
        VALUES (?, ?, ?, ?)
        ON CONFLICT (id)
        DO UPDATE SET
            amount = excluded.amount
    ";

    pub const UPSERT_NON_FUNGIBLE_ASSET: &'static str = "INSERT INTO
        non_fungible_assets (
            id,
            resource_address,
            nfids,
            last_updated,
            account_address
        )
        VALUES (?, ?, ?, ?)
        ON CONFLICT (id)
        DO UPDATE SET
            nfids = excluded.nfids
    ";

    pub const UPSERT_TRANSACTION: &'static str = "INSERT INTO
        transactions (
            id,
            timestamp,
            state_version,
            balance_changes,
            status
        )
        VALUES (?, ?, ?, ?, ?)
        ON CONFLICT (id)
        DO UPDATE SET 
            status = excluded.status
    ";
}
