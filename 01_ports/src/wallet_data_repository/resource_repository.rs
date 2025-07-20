use crate::wallet_data_repository::WalletDataRepository;

pub trait ResourceRepository: WalletDataRepository {
    type Resource;
    type ResourceID;

    fn upsert_resource(&self, resource: Self::Resource) -> Result<Self::Resource, Self::Error>;

    fn upsert_resources<Resources: IntoIterator<Item = Self::Resource>>(
        &self,
        resources: Resources,
    ) -> Result<(), Self::Error>;

    fn get_resource(&self, resource_id: Self::ResourceID) -> Result<Self::Resource, Self::Error>;

    fn get_all_resources<Resources: FromIterator<Self::Resource>>(
        &self,
    ) -> Result<Resources, Self::Error>;

    fn delete_resource(&self, resource_id: Self::ResourceID) -> Result<(), Self::Error>;
}
