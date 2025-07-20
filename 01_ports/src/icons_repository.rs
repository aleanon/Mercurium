use deps::zeroize::ZeroizeOnDrop;

pub trait IconsRepository
where
    Self: Sized,
{
    type Key: ZeroizeOnDrop;
    type Path;
    type Error: std::error::Error;

    fn initialize(path: Self::Path, key: Self::Key) -> Result<Self, Self::Error>;

    fn connect(path: Self::Path, key: Self::Key) -> Result<Self, Self::Error>;

    fn delete(path: Self::Path, key: Self::Key) -> Result<(), Self::Error>;
}
