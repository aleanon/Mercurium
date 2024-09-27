pub const CREATE_TABLE_PASSWORD_HASH: &'static str = "CREATE TABLE IF NOT EXISTS 
    password_hash (
        id INTEGER NOT NULL PRIMARY KEY,
        password TEXT NOT NULL
    )
";

pub const UPSERT_PASSWORD_HASH: &'static str = "INSERT INTO
    password_hash (
        id,
        password
    )
    VALUES (?,?)
    ON CONFLICT (id)
    DO UPDATE SET
        password = excluded.password
";
