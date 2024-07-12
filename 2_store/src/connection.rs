use super::db::DbError;
use debug_print::debug_println;
use types::{
    app_path::AppPath,
    crypto::{HexKey, Key},
    Network,
};

use rusqlite::OpenFlags;

pub fn connection_new_database(
    network: Network,
    key: &HexKey,
) -> Result<rusqlite::Connection, DbError> {
    let app_path = AppPath::get().create_directories_if_not_exists()?;

    let path = app_path.db_path_ref(network);

    debug_println!("Db path: {:?}", path);

    let conn = rusqlite::Connection::open_with_flags(
        path,
        OpenFlags::SQLITE_OPEN_CREATE | OpenFlags::SQLITE_OPEN_READ_WRITE,
    )?;

    conn.execute_batch(&format!("PRAGMA key = '{}'", key.as_str()))?;

    debug_println!("Db connection up");

    Ok(conn)
}

pub fn connection_existing_database(
    network: Network,
    key: &HexKey,
) -> Result<rusqlite::Connection, DbError> {
    let path = AppPath::get().db_path_ref(network);

    debug_println!("Db path: {:?}", path);

    if !path.exists() {
        return Err(DbError::DatabaseNotFound);
    }

    let conn = rusqlite::Connection::open_with_flags(path, OpenFlags::SQLITE_OPEN_READ_WRITE)?;

    conn.execute_batch(&format!("PRAGMA key = '{}'", key.as_str()))?;

    debug_println!("Db connection up");

    Ok(conn)
}

pub async fn async_connection_new_database(
    network: Network,
    key: HexKey,
) -> Result<tokio_rusqlite::Connection, DbError> {
    let app_path = AppPath::get().create_directories_if_not_exists()?;

    let path = app_path.db_path_ref(network);

    debug_println!("Db path: {:?}", path);

    let conn = tokio_rusqlite::Connection::open_with_flags(
        path,
        OpenFlags::SQLITE_OPEN_CREATE | OpenFlags::SQLITE_OPEN_READ_WRITE,
    )
    .await?;

    conn.call_unwrap(move |conn| conn.execute_batch(&format!("PRAGMA key = '{}'", key.as_str())))
        .await?;

    debug_println!("Async Db connection up");

    Ok(conn)
}

pub async fn async_connection_existing_database(
    network: Network,
    key: HexKey,
) -> Result<tokio_rusqlite::Connection, DbError> {
    let path = AppPath::get().db_path_ref(network);

    debug_println!("Db path: {:?}", path);

    if !path.exists() {
        return Err(DbError::DatabaseNotFound);
    }

    let conn =
        tokio_rusqlite::Connection::open_with_flags(path, OpenFlags::SQLITE_OPEN_READ_ONLY).await?;

    debug_println!("AsyncDb connection up");

    conn.call_unwrap(move |conn| conn.execute_batch(&format!("PRAGMA key = '{}'", key.as_str())))
        .await?;

    Ok(conn)
}

pub async fn open_db_read_only_async(
    network: Network,
    key: HexKey,
) -> Result<tokio_rusqlite::Connection, DbError> {
    let path = AppPath::get().db_path_ref(network);

    let conn =
        tokio_rusqlite::Connection::open_with_flags(path, OpenFlags::SQLITE_OPEN_READ_ONLY).await?;

    conn.call_unwrap(move |conn| conn.execute_batch(&format!("PRAGMA key = '{}'", key.as_str())))
        .await?;

    Ok(conn)
}
