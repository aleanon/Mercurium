use deps::zeroize::ZeroizeOnDrop;

pub trait IconsRepository
where
    Self: Sized,
{
    type Key: ZeroizeOnDrop;
    type Path;
    type Icon;
    type Error: std::error::Error;

    fn initialize(path: Self::Path, key: Self::Key) -> Result<Self, Self::Error>;

    fn load(path: Self::Path, key: Self::Key) -> Result<Self, Self::Error>;

    fn save_icon(&self, icon: Self::Icon) -> Result<(), Self::Error>;

    fn delete(path: Self::Path, key: Self::Key) -> Result<(), Self::Error>;
}
