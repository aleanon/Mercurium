use std::fmt::Display;

// use anyhow::Result;


const TRANSACTION_ID_LENGTH: usize = 30;

pub enum TransactionError {
    TxIdNonAsciiChar,
    InvalidIdLength {
        expected: usize,
        found: usize,
    }
}


#[derive(Debug, Clone, PartialEq, Eq, Ord)]
pub struct Transaction {
    pub tx_id: TransactionId,       //primary key
    pub account_id: usize, //foreign key
    pub timestamp: usize, //placeholder type
    pub fee: i32,
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
        if self.timestamp < other.timestamp {
            Some(std::cmp::Ordering::Less)
        } else if self.timestamp == other.timestamp {
            Some(std::cmp::Ordering::Equal)
        } else {Some(std::cmp::Ordering::Greater)}
    }
}

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
            Self::Success => write!(f, "SUCCESS"),
            Self::Failed => write!(f, "FAILED"),
            Self::Pending => write!(f, "PENDING"),
        }
    }
}