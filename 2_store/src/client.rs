use std::path::Path;

use super::db::DbError;
use debug_print::debug_println;
use types::{crypto::DataBaseKey, AppPath, Network};

use async_sqlite::rusqlite::{self, OpenFlags};

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
    let client = build_db_client(path).await?;

    set_database_key(&client, key).await?;

    debug_println!("AsyncDb connection up");

    Ok(client)
}

async fn build_db_client(path: &Path) -> Result<async_sqlite::Client, async_sqlite::Error> {
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
        .conn(move |conn| {
            let set_key_statement = key.set_key_statement();
            let set_key_statement_str = std::str::from_utf8(&set_key_statement)
                .map_err(|err| rusqlite::Error::Utf8Error(err))?;
            conn.execute_batch(set_key_statement_str)
        })
        .await
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
