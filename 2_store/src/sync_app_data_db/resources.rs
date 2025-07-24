use types::wallet_repository::ResourceRepository;

use crate::sync_app_data_db::SyncAppDataDb;

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

// impl ResourceRepository for SyncAppDataDb {
//     type Resource = types::Resource;
//     type ID = types::address::ResourceAddress;

//     fn upsert_resource(&self, resource: Self::Resource) -> Result<Self::Resource, Self::Error> {}
// }
