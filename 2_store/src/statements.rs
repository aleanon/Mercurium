pub mod create {
    pub const CREATE_TABLE_PASSWORD_HASH: &'static str = "CREATE TABLE IF NOT EXISTS 
        password_hash (
            id INTEGER NOT NULL PRIMARY KEY,
            password TEXT NOT NULL
        )
    ";

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

    pub const CREATE_TABLE_RESOURCES: &'static str = "CREATE TABLE IF NOT EXISTS 
        resources (
            address BLOB NOT NULL PRIMARY KEY,
            name TEXT NOT NULL,
            symbol TEXT NOT NULL,
            description TEXT NOT NULL,
            current_supply TEXT NOT NULL,
            divisibility BLOB,
            tags BLOB NOT NULL
        )
    ";

    pub const CREATE_TABLE_FUNGIBLE_ASSETS: &'static str = "CREATE TABLE IF NOT EXISTS 
        fungible_assets (
            id BLOB NOT NULL PRIMARY KEY,
            resource_address BLOB NOT NULL,
            amount TEXT NOT NULL,
            account_address BLOB NOT NULL,
            FOREIGN KEY(resource_address) REFERENCES resources(address),
            FOREIGN KEY(account_address) REFERENCES accounts(address)
        )
    ";

    pub const CREATE_TABLE_NON_FUNGIBLE_ASSETS: &'static str = "CREATE TABLE IF NOT EXISTS 
        non_fungible_assets (
            id BLOB NOT NULL PRIMARY KEY,
            resource_address BLOB NOT NULL,
            nfids BLOB NOT NULL,
            account_address BLOB NOT NULL,
            FOREIGN KEY(resource_address) REFERENCES resources(address),
            FOREIGN KEY(account_address) REFERENCES accounts(address)
        )
    ";

    pub const CREATE_TABLE_TRANSACTIONS: &'static str = "CREATE TABLE IF NOT EXISTS 
        transactions (
            id BLOB NOT NULL PRIMARY KEY,
            transaction_address BLOB NOT NULL,
            timestamp BLOB NOT NULL,
            state_version INTEGER NOT NULL,
            message TEXT
        )
    ";

    pub const CREATE_TABLE_BALANCE_CHANGES: &'static str = "CREATE TABLE IF NOT EXISTS
        balance_changes (
            id BLOB NOT NULL PRIMARY KEY,
            account BLOB NOT NULL,
            resource BLOB NOT NULL,
            nfids BLOB,
            amount TEXT,
            tx_id BLOB NOT NULL,
            FOREIGN KEY(tx_id) REFERENCES transactions(id)
        )
    ";

    pub const CREATE_TABLE_RESOURCE_IMAGES: &'static str = "CREATE TABLE IF NOT EXISTS
        resource_images (
            resource_address BLOB NOT NULL PRIMARY KEY,
            image_data BLOB NOT NULL
        )
    ";

    pub const CREATE_TABLE_NFT_IMAGES: &'static str = "CREATE TABLE IF NOT EXISTS
        nft_images (
            nfid TEXT NOT NULL PRIMARY KEY,
            image_data BLOB NOT NULL,
            resource_address BLOB NOT NULL,
            FOREIGN KEY(resource_address) REFERENCES resource_images(resource_address)
        )
    ";
}

pub mod upsert {
    pub const UPSERT_PASSWORD_HASH: &'static str = "INSERT INTO
        password_hash (
            id,
            password
        )
        VALUES (?,?)
        ON CONFLICT (id)
        DO UPDATE SET
            password = excluded.password
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

    pub const UPSERT_RESOURCE: &'static str = "INSERT INTO
        resources (
            address,
            name,
            symbol,
            description,
            current_supply,
            divisibility,
            tags
        )
        VALUES (?, ?, ?, ?, ?, ?, ?)
        ON CONFLICT (address)
        DO UPDATE SET 
            name = excluded.name,
            symbol = excluded.symbol,
            description = excluded.description,
            current_supply = excluded.current_supply,
            divisibility = excluded.divisibility,
            tags = excluded.tags
    ";

    pub const UPSERT_FUNGIBLE_ASSET: &'static str = "INSERT INTO
        fungible_assets (
            id,
            resource_address,
            amount,
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
            transaction_address,
            timestamp,
            state_version,
            status
        )
        VALUES (?, ?, ?, ?, ?)
        ON CONFLICT (id)
        DO UPDATE SET 
            status = excluded.status
    ";

    pub const UPSERT_RESOURCE_IMAGE: &'static str = "INSERT INTO
        resource_images (
            resource_address,
            image_data
        )
        VALUES (?,?)
        ON CONFLICT (resource_address)
        DO UPDATE SET
            image_data = excluded.image_data
    ";

    pub const UPSERT_NFT_IMAGE: &'static str = "INSERT INTO
        nft_images (
            nfid,
            image_data,
            resource_address
        )
        VALUES (?,?,?)
        ON CONFLICT (nfid)
        DO UPDATE SET
            image_data = excluded.image_data
    ";
}

pub mod insert {
    pub const INSERT_BALANCE_CHANGE: &'static str = "INSERT INTO
        balance_changes (
            id,
            account,
            resource,
            nfids,
            amount,
            tx_id
        )
        VALUES (?,?,?,?,?,?)

    ";
}
