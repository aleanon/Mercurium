pub const CREATE_TABLE_BALANCE_CHANGES: &'static str = "CREATE TABLE IF NOT EXISTS
    balance_changes (
        id BLOB NOT NULL PRIMARY KEY,
        account BLOB NOT NULL,
        resource BLOB NOT NULL,
        nfids BLOB,
        amount TEXT,
        tx_id BLOB NOT NULL,
        FOREIGN KEY(tx_id) REFERENCES transactions(id)
    )
";
/// Balance changes affecting a given account (the `account` column), used to assemble that
/// account's transaction history.
pub const SELECT_BALANCE_CHANGES_BY_ACCOUNT: &'static str =
    "SELECT * FROM balance_changes WHERE account = ?";

/// Balance changes belonging to a given transaction (linked by the `tx_id` foreign key).
pub const SELECT_BALANCE_CHANGES_BY_TX_ID: &'static str =
    "SELECT * FROM balance_changes WHERE tx_id = ?";

pub const INSERT_BALANCE_CHANGE: &'static str = "INSERT OR REPLACE INTO
    balance_changes (
        id,
        account,
        resource,
        nfids,
        amount,
        tx_id
    )
    VALUES (?,?,?,?,?,?)
";
