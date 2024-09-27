pub const CREATE_TABLE_TRANSACTIONS: &'static str = "CREATE TABLE IF NOT EXISTS 
    transactions (
        id BLOB NOT NULL PRIMARY KEY,
        transaction_address BLOB NOT NULL,
        timestamp BLOB NOT NULL,
        state_version INTEGER NOT NULL,
        message TEXT
    )
";

pub const UPSERT_TRANSACTION: &'static str = "INSERT INTO
    transactions (
        id,
        transaction_address,
        timestamp,
        state_version,
        status
    )
    VALUES (?, ?, ?, ?, ?)
    ON CONFLICT (id)
    DO UPDATE SET 
        status = excluded.status
";
