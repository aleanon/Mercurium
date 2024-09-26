use std::path::Path;

use debug_print::debug_println;
use types::{crypto::DataBaseKey, AppPath, Network};

use async_sqlite::rusqlite::OpenFlags;

use crate::db::DbError;

pub(crate) async fn main_db_client(
    network: Network,
    key: DataBaseKey,
) -> Result<async_sqlite::Client, DbError> {
    let app_path = AppPath::get().create_directories_if_not_exists()?;
    let path = app_path.db_path_ref(network);

    debug_println!("Db path: {:?}", path);

    async_client(path, key).await
}

pub(crate) async fn iconcache_client(
    network: Network,
    key: DataBaseKey,
) -> Result<async_sqlite::Client, DbError> {
    let app_path = AppPath::get();
    let path = app_path.icon_cache_ref(network);

    debug_println!("IconCache path: {:?}", path);

    async_client(path, key).await
}

async fn async_client(path: &Path, key: DataBaseKey) -> Result<async_sqlite::Client, DbError> {
    let client = build_async_db_client(path).await?;
    set_database_key(&client, key).await?;

    debug_println!("AsyncDb connection up");

    Ok(client)
}

async fn build_async_db_client(path: &Path) -> Result<async_sqlite::Client, async_sqlite::Error> {
    async_sqlite::ClientBuilder::new()
        .path(path)
        .flags(OpenFlags::SQLITE_OPEN_CREATE | OpenFlags::SQLITE_OPEN_READ_WRITE)
        .open()
        .await
}

async fn set_database_key(
    client: &async_sqlite::Client,
    key: DataBaseKey,
) -> Result<(), async_sqlite::Error> {
    client
        .conn(move |conn| conn.pragma_update(None, "key", key))
        .await
}

#[cfg(test)]
mod test {
    use std::{fs::File, io::Write};

    use types::crypto::Password;

    use crate::statements;

    use super::*;

    #[tokio::test]
    async fn test_set_database_key() {
        File::create("./mock.db").unwrap().write(&[]).unwrap();

        let key = Password::from("SomePasswordtype")
            .derive_new_db_encryption_key()
            .unwrap()
            .0;
        {
            let client = async_sqlite::ClientBuilder::new()
                .path("./mock.db")
                .open()
                .await
                .expect("Failed to open in memory database");

            set_database_key(&client, key.clone())
                .await
                .expect("Failed to set database key");

            client
                .conn(|conn| conn.execute(statements::create::CREATE_TABLE_ACCOUNTS, []))
                .await
                .expect("Unable to create table, accounts");
        }
        let second_client = async_sqlite::ClientBuilder::new()
            .path("./mock.db")
            .open()
            .await
            .expect("Failed to open second client");

        let query = second_client
            .conn(|conn| conn.execute(&statements::create::CREATE_TABLE_FUNGIBLE_ASSETS, []))
            .await;
        assert!(query.is_err());

        set_database_key(&second_client, key)
            .await
            .expect("Failed to set database key for second client");

        second_client
            .conn(|conn| conn.execute(&statements::create::CREATE_TABLE_NON_FUNGIBLE_ASSETS, []))
            .await
            .expect("Unable to create table, fungibles");
    }
}

// pub async fn async_connection_existing_database(
//     network: Network,
//     key: DataBaseKey,
// ) -> Result<async_sqlite::Client, DbError> {
//     let path = AppPath::get().db_path_ref(network);

//     debug_println!("Db path: {:?}", path);

//     if !path.exists() {
//         return Err(DbError::DatabaseNotFound);
//     }

//     let client = async_sqlite::ClientBuilder::new()
//         .path(path)
//         .flags(OpenFlags::SQLITE_OPEN_READ_WRITE)
//         .open()
//         .await?;

//     debug_println!("AsyncDb connection up");

//     client
//         .conn(move |conn| {
//             let set_key_statement = key.set_key_statement();
//             let set_key_statement_str = std::str::from_utf8(&set_key_statement)
//                 .map_err(|err| rusqlite::Error::Utf8Error(err))?;
//             conn.execute_batch(set_key_statement_str)
//         })
//         .await?;

//     Ok(client)
// }
