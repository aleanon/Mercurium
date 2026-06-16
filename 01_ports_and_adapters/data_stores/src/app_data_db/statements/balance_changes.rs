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
pub const INSERT_BALANCE_CHANGE: &'static str = "INSERT INTO
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
