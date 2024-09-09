use super::db::DbError;
use debug_print::debug_println;
use types::{crypto::DataBaseKey, AppPath, Network};

use rusqlite::OpenFlags;

pub fn connection_new_database(
    network: Network,
    key: &DataBaseKey,
) -> Result<rusqlite::Connection, DbError> {
    let app_path = AppPath::get().create_directories_if_not_exists()?;

    let path = app_path.db_path_ref(network);

    debug_println!("Db path: {:?}", path);

    let conn = rusqlite::Connection::open_with_flags(
        path,
        OpenFlags::SQLITE_OPEN_CREATE | OpenFlags::SQLITE_OPEN_READ_WRITE,
    )?;

    let set_key_statement = key.set_key_statement();
    let set_key_statement_str =
        std::str::from_utf8(&set_key_statement).map_err(|err| rusqlite::Error::Utf8Error(err))?;

    conn.execute_batch(set_key_statement_str)?;

    debug_println!("Db connection up");

    Ok(conn)
}

pub fn connection_existing_database(
    network: Network,
    key: &DataBaseKey,
) -> Result<rusqlite::Connection, DbError> {
    let path = AppPath::get().db_path_ref(network);

    debug_println!("Db path: {:?}", path);

    if !path.exists() {
        return Err(DbError::DatabaseNotFound);
    }

    let conn = rusqlite::Connection::open_with_flags(path, OpenFlags::SQLITE_OPEN_READ_WRITE)?;

    let set_key_statement = key.set_key_statement();
    let set_key_statement_str =
        std::str::from_utf8(&set_key_statement).map_err(|err| rusqlite::Error::Utf8Error(err))?;

    conn.execute_batch(set_key_statement_str)?;

    debug_println!("Db connection up");

    Ok(conn)
}

pub async fn async_connection(
    network: Network,
    key: DataBaseKey,
) -> Result<async_sqlite::Client, DbError> {
    let app_path = AppPath::get().create_directories_if_not_exists()?;

    let path = app_path.db_path_ref(network);

    debug_println!("Db path: {:?}", path);

    let client = async_sqlite::ClientBuilder::new()
        .path(path)
        .flags(OpenFlags::SQLITE_OPEN_CREATE | OpenFlags::SQLITE_OPEN_READ_WRITE)
        .open()
        .await?;

    client
        .conn(move |conn| {
            let set_key_statement = key.set_key_statement();
            let set_key_statement_str = std::str::from_utf8(&set_key_statement)
                .map_err(|err| rusqlite::Error::Utf8Error(err))?;
            conn.execute_batch(set_key_statement_str)
        })
        .await?;

    debug_println!("AsyncDb connection up");

    Ok(client)
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
