use deps::*;

pub enum Error {
    FailedToSaveSecret,
    FailedToDeleteSecret,
    FailedToRetrieveSecret,
    FailedToDecryptSecret,
}

pub trait SecretsRepository {
    type Secret: encrypt::traits::Encrypt;
    type Key: encrypt::traits::Key;

    fn delete(&self) -> Result<(), Error>;

    fn save(&self, secret: Self::Secret) -> Result<(), Error>;

    fn get(&self) -> Result<Self::Secret, Error>;
}
