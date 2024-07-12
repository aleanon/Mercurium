use std::num::NonZeroU32;

use ring::pbkdf2::{self, PBKDF2_HMAC_SHA512};
use rusqlite::{types::FromSql, ToSql};
use zeroize::ZeroizeOnDrop;

use crate::{
    crypto::{Password, Salt},
    debug_info,
    unwrap_unreachable::UnwrapUnreachable,
};

#[derive(PartialEq, Eq, ZeroizeOnDrop)]
pub struct HashedPassword([u8; Self::LENGTH]);

impl HashedPassword {
    const LENGTH: usize = 64;
    const HASH_ITERATIONS: u32 = 50000;

    pub fn db_key_hash(salt: &Salt, password: &Password) -> Self {
        let mut hash = [0u8; Self::LENGTH];
        let iterations = NonZeroU32::new(Self::HASH_ITERATIONS)
            .unwrap_unreachable(debug_info!("Nonzero value for password hash iterations"));

        pbkdf2::derive(
            PBKDF2_HMAC_SHA512,
            iterations,
            salt.as_bytes(),
            password.as_str().as_bytes(),
            &mut hash,
        );

        Self(hash)
    }
}

impl ToSql for HashedPassword {
    fn to_sql(&self) -> rusqlite::Result<rusqlite::types::ToSqlOutput<'_>> {
        Ok(rusqlite::types::ToSqlOutput::Borrowed(
            rusqlite::types::ValueRef::Blob(&self.0),
        ))
    }
}

impl FromSql for HashedPassword {
    fn column_result(value: rusqlite::types::ValueRef<'_>) -> rusqlite::types::FromSqlResult<Self> {
        let blob = value.as_blob()?;
        Ok(Self(blob.try_into().map_err(|_| {
            rusqlite::types::FromSqlError::InvalidBlobSize {
                expected_size: Self::LENGTH,
                blob_size: blob.len(),
            }
        })?))
    }
}
