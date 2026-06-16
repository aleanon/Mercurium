// SQLite store implementation (the encrypted wallet + icons databases), relocated from the
// former standalone `store` crate.
mod app_data_db;
mod database;
mod icons_db;
mod sqlite_key;

pub mod adapters;
pub mod ports;

pub use app_data_db::AppDataDb;
pub use database::{DataBase, DbError};
pub use icons_db::{IconsDb, SyncIconsDb};
pub use sqlite_key::SqliteKey;

// Settings store (JSON file).
pub use adapters::JsonSettingsStore;
pub use adapters::JsonProfileStore;
pub use ports::profile_store::ProfileStore;
pub use ports::settings_store::SettingsStore;
