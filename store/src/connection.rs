use super::db::DbError;
use debug_print::debug_println;
use types::{app_path::AppPath, Network};

use rusqlite::OpenFlags;
// use anyhow::{Result, Context};

#[cfg(windows)]
pub fn connection_new_database(network: Network) -> Result<rusqlite::Connection, DbError> {
    let app_path = AppPath::get().create_directories_if_not_exists()?;

    let path = app_path.db_path_ref(network);

    debug_println!("Db path: {:?}", path);

    let conn = rusqlite::Connection::open_with_flags(
        path,
        OpenFlags::SQLITE_OPEN_CREATE | OpenFlags::SQLITE_OPEN_READ_WRITE,
    )?;

    debug_println!("Db connection up");

    //conn.execute_batch(&format!("PRAGMA key = '{}'", "MyPassPhrase")).unwrap();

    Ok(conn)
}

#[cfg(windows)]
pub fn connection_existing_database(network: Network) -> Result<rusqlite::Connection, DbError> {
    let path = AppPath::get().db_path_ref(network);

    debug_println!("Db path: {:?}", path);

    if !path.exists() {
        return Err(DbError::DatabaseNotFound);
    }

    let conn = rusqlite::Connection::open_with_flags(path, OpenFlags::SQLITE_OPEN_READ_WRITE)?;
    //.context("Unable to connect to database")?;

    debug_println!("Db connection up");

    //conn.execute_batch(&format!("PRAGMA key = '{}'", "MyPassPhrase")).unwrap();

    Ok(conn)
}

#[cfg(windows)]
pub async fn async_connection_new_database(
    network: Network,
) -> Result<tokio_rusqlite::Connection, DbError> {
    let app_path = AppPath::get().create_directories_if_not_exists()?;

    let path = app_path.db_path_ref(network);

    debug_println!("Db path: {:?}", path);

    let conn = tokio_rusqlite::Connection::open_with_flags(
        path,
        OpenFlags::SQLITE_OPEN_CREATE | OpenFlags::SQLITE_OPEN_READ_WRITE,
    )
    .await?;

    debug_println!("Async Db connection up");

    //conn.execute_batch(&format!("PRAGMA key = '{}'", "MyPassPhrase")).unwrap();

    Ok(conn)
}

#[cfg(windows)]
pub async fn async_connection_existing_database(
    network: Network,
) -> Result<tokio_rusqlite::Connection, DbError> {
    let path = AppPath::get().db_path_ref(network);

    debug_println!("Db path: {:?}", path);

    if !path.exists() {
        return Err(DbError::DatabaseNotFound);
    }

    let conn =
        tokio_rusqlite::Connection::open_with_flags(path, OpenFlags::SQLITE_OPEN_READ_ONLY).await?;
    //.context("Unable to connect to database")?;

    debug_println!("AsyncDb connection up");

    //conn.execute_batch(&format!("PRAGMA key = '{}'", "MyPassPhrase")).unwrap();

    Ok(conn)
}

pub async fn open_db_read_only_async(
    network: Network,
) -> Result<tokio_rusqlite::Connection, DbError> {
    let path = AppPath::get().db_path_ref(network);

    let connection =
        tokio_rusqlite::Connection::open_with_flags(path, OpenFlags::SQLITE_OPEN_READ_ONLY).await?;
    Ok(connection)
}
