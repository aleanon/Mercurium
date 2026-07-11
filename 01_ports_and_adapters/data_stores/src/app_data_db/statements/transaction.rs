pub const CREATE_TABLE_TRANSACTIONS: &str = "CREATE TABLE IF NOT EXISTS 
    transactions (
        id BLOB NOT NULL PRIMARY KEY,
        transaction_address BLOB NOT NULL,
        timestamp BLOB NOT NULL,
        state_version INTEGER NOT NULL,
        message TEXT
    )
";

/// A single transaction by its primary-key `id`.
pub const SELECT_TRANSACTION_BY_ID: &str = "SELECT * FROM transactions WHERE id = ?";

pub const UPSERT_TRANSACTION: &str = "INSERT INTO
    transactions (
        id,
        transaction_address,
        timestamp,
        state_version,
        message
    )
    VALUES (?, ?, ?, ?, ?)
    ON CONFLICT (id)
    DO UPDATE SET
        transaction_address = excluded.transaction_address,
        timestamp = excluded.timestamp,
        state_version = excluded.state_version,
        message = excluded.message
";
