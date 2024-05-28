use std::{
    collections::{BTreeMap, HashMap},
    fmt::Display,
};

use serde::{Deserialize, Serialize};

use crate::{
    address::transaction_address::TransactionAddress, debug_info,
    unwrap_unreachable::UnwrapUnreachable, AccountAddress, Address, Decimal, ResourceAddress,
};

// use anyhow::Result;

pub enum TransactionError {
    TxIdNonAsciiChar,
    InvalidIdLength { expected: usize, found: usize },
}

#[derive(Debug, Clone)]
pub struct Transaction {
    pub id: TransactionId, //primary key
    pub address: TransactionAddress,
    pub timestamp: TimeStamp,
    pub state_version: u64,
    pub withdraws: Vec<(ResourceAddress, String)>,
    pub deposits: Vec<(ResourceAddress, String)>,
}

impl Transaction {
    pub fn new(
        timestamp: TimeStamp,
        state_version: u64,
        withdraws: Vec<(ResourceAddress, String)>,
        account_address: &AccountAddress,
        transaction_address: TransactionAddress,
    ) -> Self {
        Self {
            id: TransactionId::new(&account_address, &transaction_address),
            address: transaction_address,
            timestamp,
            state_version,
            withdraws,
            deposits: Vec::new(),
        }
    }
}

impl PartialOrd for Transaction {
    fn ge(&self, other: &Self) -> bool {
        self.timestamp >= other.timestamp
    }

    fn gt(&self, other: &Self) -> bool {
        self.timestamp > other.timestamp
    }

    fn le(&self, other: &Self) -> bool {
        self.timestamp <= other.timestamp
    }

    fn lt(&self, other: &Self) -> bool {
        self.timestamp < other.timestamp
    }

    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Transaction {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        if self.timestamp == other.timestamp {
            self.id.cmp(&self.id)
        } else if self.timestamp < other.timestamp {
            std::cmp::Ordering::Less
        } else {
            std::cmp::Ordering::Greater
        }
    }
}

impl PartialEq for Transaction {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }

    fn ne(&self, other: &Self) -> bool {
        self.id != other.id
    }
}

impl Eq for Transaction {}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct TransactionId([u8; Self::LENGTH]);

impl TransactionId {
    pub const CHECKSUM_LEN: usize = 6;
    const CHECKSUM_START_INDEX: usize = Self::LENGTH - Self::CHECKSUM_LEN;
    const FROM_ACCOUNT_ADDRESS_LENGTH: usize = AccountAddress::CHECKSUM_LENGTH * 2;
    const LENGTH: usize =
        AccountAddress::CHECKSUM_LENGTH * 2 + TransactionAddress::CHECKSUM_LENGTH * 2;
    const LENGTH_HALF: usize = Self::LENGTH / 2;

    pub fn new(account_address: &AccountAddress, tx_address: &TransactionAddress) -> Self {
        let mut transaction_id = [0u8; Self::LENGTH];
        transaction_id[..Self::FROM_ACCOUNT_ADDRESS_LENGTH]
            .copy_from_slice(&account_address.checksum_double());
        transaction_id[Self::FROM_ACCOUNT_ADDRESS_LENGTH..]
            .copy_from_slice(&tx_address.checksum_double());
        Self(transaction_id)
    }

    pub fn checksum(&self) -> [u8; Self::CHECKSUM_LEN] {
        self.0[Self::CHECKSUM_START_INDEX..]
            .try_into()
            .unwrap_unreachable(debug_info!("Invalid checksum length"))
    }

    pub fn checksum_slice(&self) -> &[u8] {
        &self.0[Self::CHECKSUM_START_INDEX..]
    }
}

impl rusqlite::types::FromSql for TransactionId {
    fn column_result(value: rusqlite::types::ValueRef<'_>) -> rusqlite::types::FromSqlResult<Self> {
        match value {
            rusqlite::types::ValueRef::Blob(slice) => Ok(Self(slice.try_into().map_err(|_| {
                rusqlite::types::FromSqlError::InvalidBlobSize {
                    expected_size: Self::LENGTH,
                    blob_size: slice.len(),
                }
            })?)),
            _ => Err(rusqlite::types::FromSqlError::InvalidType),
        }
    }
}

impl rusqlite::types::ToSql for TransactionId {
    fn to_sql(&self) -> rusqlite::Result<rusqlite::types::ToSqlOutput<'_>> {
        Ok(rusqlite::types::ToSqlOutput::Borrowed(
            rusqlite::types::ValueRef::Blob(self.0.as_slice()),
        ))
    }
}

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Clone)]
pub struct TimeStamp([u8; Self::LENGTH]);

impl TimeStamp {
    const LENGTH: usize = 7;

    pub fn new(year: u16, month: u8, day: u8, hour: u8, minute: u8, second: u8) -> Self {
        let year_high = (year / 100) as u8;
        let year_low = (year % 100) as u8;
        Self([year_high, year_low, month, day, hour, minute, second])
    }

    pub fn as_slice(&self) -> &[u8] {
        &self.0
    }

    pub fn year(&self) -> u16 {
        self.0[0] as u16 * 100 + self.0[1] as u16
    }

    pub fn to_string(&self) -> String {
        format!(
            "{}{}-{}-{} {}:{}:{}",
            self.0[0], self.0[1], self.0[2], self.0[3], self.0[4], self.0[5], self.0[6]
        )
    }
}

impl std::fmt::Display for TimeStamp {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}{}-{}-{} {}:{}:{}",
            self.0[0], self.0[1], self.0[2], self.0[3], self.0[4], self.0[5], self.0[6]
        )
    }
}

impl rusqlite::types::FromSql for TimeStamp {
    fn column_result(value: rusqlite::types::ValueRef<'_>) -> rusqlite::types::FromSqlResult<Self> {
        let slice = value.as_blob()?;
        Ok(Self(slice.try_into().map_err(|_| {
            rusqlite::types::FromSqlError::InvalidBlobSize {
                expected_size: Self::LENGTH,
                blob_size: slice.len(),
            }
        })?))
    }
}

impl rusqlite::types::ToSql for TimeStamp {
    fn to_sql(&self) -> rusqlite::Result<rusqlite::types::ToSqlOutput<'_>> {
        Ok(rusqlite::types::ToSqlOutput::Borrowed(
            rusqlite::types::ValueRef::Blob(self.0.as_slice()),
        ))
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum TransactionStatus {
    Failed,
    Success,
    Pending,
}

impl rusqlite::types::FromSql for TransactionStatus {
    fn column_result(value: rusqlite::types::ValueRef<'_>) -> rusqlite::types::FromSqlResult<Self> {
        match value {
            rusqlite::types::ValueRef::Integer(int) => match int {
                0 => Ok(TransactionStatus::Failed),
                1 => Ok(TransactionStatus::Success),
                2 => Ok(TransactionStatus::Pending),
                n => Err(rusqlite::types::FromSqlError::OutOfRange(n)),
            },
            _ => Err(rusqlite::types::FromSqlError::InvalidType),
        }
    }
}

impl Display for TransactionStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Success => write!(f, "Success"),
            Self::Failed => write!(f, "Failed"),
            Self::Pending => write!(f, "Pending"),
        }
    }
}

pub struct BalanceChange {
    id: BalanceChangeId,
    account: AccountAddress,
    resource: ResourceAddress,
    amount: String,
}

impl BalanceChange {
    pub fn new(
        transaction: TransactionId,
        account: AccountAddress,
        resource: ResourceAddress,
        amount: String,
    ) -> Self {
        Self {
            id: BalanceChangeId::new(
                transaction.checksum_slice(),
                account.checksum_slice(),
                resource.checksum_slice(),
            ),
            account,
            resource,
            amount,
        }
    }
}

pub struct BalanceChangeId([u8; Self::LENGTH]);

impl BalanceChangeId {
    const LENGTH: usize = TransactionId::CHECKSUM_LEN
        + AccountAddress::CHECKSUM_LENGTH
        + ResourceAddress::CHECKSUM_LEN;

    const LAST_CHECKSUM_START: usize =
        TransactionId::CHECKSUM_LEN + AccountAddress::CHECKSUM_LENGTH;

    pub fn new(tx_checksum: &[u8], account_checksum: &[u8], resource_checksum: &[u8]) -> Self {
        let mut id = [0u8; Self::LENGTH];

        id[..TransactionId::CHECKSUM_LEN].copy_from_slice(tx_checksum);
        id[TransactionId::CHECKSUM_LEN..AccountAddress::CHECKSUM_LENGTH]
            .copy_from_slice(account_checksum);
        id[Self::LAST_CHECKSUM_START..].copy_from_slice(resource_checksum);

        Self(id)
    }
}

impl rusqlite::types::FromSql for BalanceChangeId {
    fn column_result(value: rusqlite::types::ValueRef<'_>) -> rusqlite::types::FromSqlResult<Self> {
        let slice = value.as_blob()?;
        Ok(Self(slice.try_into().map_err(|_| {
            rusqlite::types::FromSqlError::InvalidBlobSize {
                expected_size: Self::LENGTH,
                blob_size: slice.len(),
            }
        })?))
    }
}

impl rusqlite::types::ToSql for BalanceChangeId {
    fn to_sql(&self) -> rusqlite::Result<rusqlite::types::ToSqlOutput<'_>> {
        Ok(rusqlite::types::ToSqlOutput::Borrowed(
            rusqlite::types::ValueRef::Blob(self.0.as_slice()),
        ))
    }
}

#[cfg(test)]
mod test {

    use super::*;

    #[test]
    fn test_timestamp() {
        let timestamp = TimeStamp::new(2024, 3, 7, 14, 40, 35);

        let string = timestamp.to_string();

        let target = String::from("2024-3-7 14:40:35");

        assert_eq!(string, target)
    }
}
