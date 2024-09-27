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
