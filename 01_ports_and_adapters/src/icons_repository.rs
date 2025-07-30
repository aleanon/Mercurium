use async_trait::async_trait;
use deps::zeroize::ZeroizeOnDrop;

#[async_trait]
pub trait IconsRepository
where
    Self: Sized,
{
    type Key: ZeroizeOnDrop;
    type Path;
    type Icon;
    type IconId;
    type Error: std::error::Error;

    async fn init_repository(path: Self::Path, key: Self::Key) -> Result<Self, Self::Error>;

    async fn load(path: Self::Path, key: Self::Key) -> Result<Self, Self::Error>;

    async fn save_icon(&self, icon: Self::Icon) -> Result<(), Self::Error>;

    async fn save_icons(
        &self,
        icons: impl IntoIterator<Item = Self::Icon>,
    ) -> Result<(), Self::Error>;

    async fn load_icon(&self, id: Self::IconId) -> Result<Self::Icon, Self::Error>;

    async fn load_icons(
        &self,
        ids: impl IntoIterator<Item = Self::IconId>,
    ) -> Result<Vec<Self::Icon>, Self::Error>;

    async fn load_all_icons(&self) -> Result<impl FromIterator<Self::Icon>, Self::Error>;

    async fn delete(path: Self::Path, key: Self::Key) -> Result<(), Self::Error>;
}
