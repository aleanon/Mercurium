use crate::{
    adapters::data_store::sqlite::Sqlite,
    ports::wallet_data_store::{Resource, ResourceId, ResourceStore, WalletDataStoreError},
};
use async_trait::async_trait;

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

#[async_trait]
impl ResourceStore for Sqlite {
    async fn upsert_resource(&self, resource: Resource) -> Result<Resource, WalletDataStoreError> {
        todo!()
    }

    async fn upsert_resources<Resources>(
        &self,
        resources: Resources,
    ) -> Result<(), WalletDataStoreError>
    where
        Resources: IntoIterator<Item = Resource> + Send + 'static,
    {
        todo!()
    }

    async fn get_resource(
        &self,
        resource_id: ResourceId,
    ) -> Result<Resource, WalletDataStoreError> {
        todo!()
    }

    async fn get_all_resources<Resources>(&self) -> Result<Resources, WalletDataStoreError>
    where
        Resources: FromIterator<Resource> + Send + 'static,
    {
        todo!()
    }

    async fn delete_resource(&self, resource_id: ResourceId) -> Result<(), WalletDataStoreError> {
        todo!()
    }
}
