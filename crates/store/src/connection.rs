use super::db::DbError;
use handles::filesystem::app_path::AppPath;
use debug_print::debug_println;

use rusqlite::OpenFlags;
// use anyhow::{Result, Context};

#[cfg(windows)]
pub fn connection_new_database() -> Result<rusqlite::Connection, DbError> {
    let app_path = AppPath::new()?.create_directories_if_not_exists()?;

    let path = app_path.db_path();

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
pub fn connection_existing_database() -> Result<rusqlite::Connection, DbError> {
    let path = AppPath::new()?.db_path();

    debug_println!("Db path: {:?}", path);

    if !path.exists() {
        return Err(DbError::DatabaseNotFound);
    }

    let conn = rusqlite::Connection::open_with_flags(
        path,
        OpenFlags::SQLITE_OPEN_READ_WRITE,
    )?;
    //.context("Unable to connect to database")?;

    debug_println!("Db connection up");

    //conn.execute_batch(&format!("PRAGMA key = '{}'", "MyPassPhrase")).unwrap();

    Ok(conn)
}

#[cfg(windows)]
pub async fn async_connection_new_database() -> Result<tokio_rusqlite::Connection, DbError> {
    let app_path = AppPath::new()?.create_directories_if_not_exists()?;

    let path = app_path.db_path();

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
pub async fn async_connection_existing_database() -> Result<tokio_rusqlite::Connection, DbError> {
    let path = AppPath::new()?.db_path();

    debug_println!("Db path: {:?}", path);

    if !path.exists() {
        return Err(DbError::DatabaseNotFound);
    }

    let conn = tokio_rusqlite::Connection::open_with_flags(
        path,
        OpenFlags::SQLITE_OPEN_READ_ONLY,
    )
    .await?;
    //.context("Unable to connect to database")?;

    debug_println!("AsyncDb connection up");

    //conn.execute_batch(&format!("PRAGMA key = '{}'", "MyPassPhrase")).unwrap();

    Ok(conn)
}

pub async fn open_db_read_only_async() -> Result<tokio_rusqlite::Connection, DbError> {
    let app_path = AppPath::new()?.db_path();

    let connection =
        tokio_rusqlite::Connection::open_with_flags(app_path, OpenFlags::SQLITE_OPEN_READ_ONLY)
            .await?;
    Ok(connection)
}

#[cfg(target_os = "linux")]
pub fn open_new_database() -> Result<Connection, DbError> {
    let mut path = match std::env::var_os("LOCALAPPDATA") {
        Some(path) => {
            let mut path = std::path::PathBuf::from(path);
            path.push("RaVault");
            path.push("database");
            path
        }
        None => {
            let mut path =
                std::env::current_exe().map_err(|err| DbError::UnableToEstablishPath(err))?;
            // .context("Failed to resolve database path")?;
            path.push("database");
            path
        }
    };

    // path.push("RaVault");

    debug_println!("Db path: {:?}", path);

    if !path.exists() {
        debug_println!("Path does not exist, creating path");

        std::fs::DirBuilder::new().create(&path)?;
        //.context("Unable to create database directory")?;

        debug_println!("Path successfully created")
    }

    path.push("appdata");
    path.set_extension("db");

    let conn = Connection::open_with_flags(
        path,
        OpenFlags::SQLITE_OPEN_CREATE | OpenFlags::SQLITE_OPEN_READ_WRITE,
    )?;
    // .context("Unable to connect to database")?;

    debug_println!("Db connection up");

    //conn.execute_batch(&format!("PRAGMA key = '{}'", "MyPassPhrase")).unwrap();

    Ok(conn)
}

#[cfg(target_os = "linux")]
pub fn open_database() -> Result<Connection, DbError> {
    let mut path = match std::env::var_os("APPDATA") {
        Some(path) => {
            let mut path = std::path::PathBuf::from(path);
            path.push("RaVault");

            path
        }
        None => {
            let path =
                std::env::current_exe().map_err(|err| DbError::UnableToEstablishPath(err))?;

            path
        }
    };

    path.push("database");
    path.push("appdata");
    path.set_extension("db");

    // path.push("RaVault");

    debug_println!("Db path: {:?}", path);

    if !path.exists() {
        return Err(DbError::DatabaseNotFound);
    }

    let conn = Connection::open_with_flags(
        path,
        OpenFlags::SQLITE_OPEN_CREATE | OpenFlags::SQLITE_OPEN_READ_WRITE,
    )?;
    //.context("Unable to connect to database")?;

    debug_println!("Db connection up");

    //conn.execute_batch(&format!("PRAGMA key = '{}'", "MyPassPhrase")).unwrap();

    Ok(conn)
}
