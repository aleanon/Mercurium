pub const CREATE_TABLE_PERSONAS: &str = "CREATE TABLE IF NOT EXISTS
    personas (
        identity_address TEXT NOT NULL PRIMARY KEY,
        id INTEGER NOT NULL,
        label TEXT NOT NULL,
        network INTEGER NOT NULL,
        derivation_path BLOB NOT NULL,
        public_key BLOB NOT NULL,
        persona_data BLOB NOT NULL,
        hidden BOOL NOT NULL
    )
";

pub const UPSERT_PERSONA: &str = "INSERT INTO
    personas (
        identity_address,
        id,
        label,
        network,
        derivation_path,
        public_key,
        persona_data,
        hidden
    )
    VALUES (?, ?, ?, ?, ?, ?, ?, ?)
    ON CONFLICT (identity_address)
    DO UPDATE SET
        id = excluded.id,
        label = excluded.label,
        network = excluded.network,
        derivation_path = excluded.derivation_path,
        public_key = excluded.public_key,
        persona_data = excluded.persona_data,
        hidden = excluded.hidden
";
