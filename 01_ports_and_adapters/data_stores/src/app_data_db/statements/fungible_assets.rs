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
