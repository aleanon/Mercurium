use std::{collections::{BTreeMap, HashMap}, fmt::Display};

use serde::{Deserialize, Serialize};

use crate::{AccountAddress, Decimal, ResourceAddress};




// use anyhow::Result;


const TRANSACTION_ID_LENGTH: usize = 30;

pub enum TransactionError {
    TxIdNonAsciiChar,
    InvalidIdLength {
        expected: usize,
        found: usize,
    }
}


#[derive(Debug, Clone)]
pub struct Transaction {
    pub tx_id: TransactionId,       //primary key 
    pub timestamp: usize, //placeholder type
    pub state_version: u64,
    pub balance_changes: HashMap<AccountAddress, BTreeMap<ResourceAddress, Decimal>>,
    pub status: TransactionStatus
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
    fn cmp(&self, other: &Self) -> scrypto::prelude::rust::cmp::Ordering {
        if self.timestamp == other.timestamp {
            self.tx_id.cmp(&self.tx_id)
        } else if self.timestamp < other.timestamp {
            std::cmp::Ordering::Less
        } else {
            std::cmp::Ordering::Greater
        }
    }
}

impl PartialEq for Transaction {
    fn eq(&self, other: &Self) -> bool {
        self.tx_id == other.tx_id
    }

    fn ne(&self, other: &Self) -> bool {
        self.tx_id != other.tx_id
    }
}

impl Eq for Transaction {}

#[derive(Debug,Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct TransactionId([u8; TRANSACTION_ID_LENGTH]);

impl TransactionId {
    pub fn from_str(s: &str) -> Result<Self, TransactionError> {
        if !s.is_ascii() {
            return Err(TransactionError::TxIdNonAsciiChar)
        }
        Ok(Self(
            s.as_bytes().try_into()
                .map_err(|_| TransactionError::InvalidIdLength { 
                    expected: TRANSACTION_ID_LENGTH, 
                    found: s.len()}
                )?
            ))
    }
}

impl rusqlite::types::FromSql for TransactionId {
    fn column_result(value: rusqlite::types::ValueRef<'_>) -> rusqlite::types::FromSqlResult<Self> {
        match value {
            rusqlite::types::ValueRef::Blob(slice) => {
                Ok(
                    Self(slice.try_into().map_err(|err| rusqlite::types::FromSqlError::Other(Box::new(err)))?)
                )
            }
            _ => Err(rusqlite::types::FromSqlError::InvalidType)
        }
    }
}

impl rusqlite::types::ToSql for TransactionId {
    fn to_sql(&self) -> rusqlite::Result<rusqlite::types::ToSqlOutput<'_>> {
        Ok(
            rusqlite::types::ToSqlOutput::Borrowed(
                rusqlite::types::ValueRef::Blob(self.0.as_ref())
            )
        )
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Timestamp {
    year: u16,
    month: u8,
    day: u8,
    hour: u8,
    minute: u8,
    second: u8,
}

impl Timestamp {
    pub fn new(year:u16, month:u8, day:u8, hour:u8, minute:u8, second:u8) -> Self {
        Self {year, month, day, hour, minute, second}
    }

    pub fn as_array(&self) -> [u8;7] {
        let year = self.year.to_be_bytes();
        [year[0], year[1], self.month, self.day, self.hour, self.minute, self.second]
    }

}



impl std::fmt::Display for Timestamp {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}-{}-{} {}:{}:{}", self.year, self.month, self.day, self.hour, self.minute, self.second)
    }
}


impl rusqlite::types::FromSql for Timestamp {
    fn column_result(value: rusqlite::types::ValueRef<'_>) -> rusqlite::types::FromSqlResult<Self> {
        match value {
            rusqlite::types::ValueRef::Blob(slice) => {
                let year = u16::from_be_bytes(slice[0..2]
                    .try_into()
                    .map_err(|_| rusqlite::types::FromSqlError::InvalidBlobSize { expected_size: 2, blob_size: slice.len() })?
                );
                Ok(Self{year, month: slice[2], day: slice[3], hour: slice[4], minute: slice[5], second: slice[6]})
            },
            _ => Err(rusqlite::types::FromSqlError::InvalidType)
        }
    }
}

impl rusqlite::types::ToSql for Timestamp {
    fn to_sql(&self) -> rusqlite::Result<rusqlite::types::ToSqlOutput<'_>> {
        Ok(rusqlite::types::ToSqlOutput::Owned(
            rusqlite::types::Value::Blob(self.as_array().to_vec())
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
                n => Err(rusqlite::types::FromSqlError::OutOfRange(n))
            }
            _ => Err(rusqlite::types::FromSqlError::InvalidType)
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

#[cfg(test)]
mod test {
    

    use super::*;

    #[test]
    fn test_timestamp() {
        let timestamp = Timestamp::new(2024, 3, 7, 14, 40, 35);

        let string = format!("{timestamp}");

        let target = String::from("2024-3-7 14:40:35");

        assert_eq!(string, target)
    }
}