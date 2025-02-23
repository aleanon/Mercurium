use store::{AppDataDb, IconsDb};
use types::crypto::Password;

pub struct DatabaseHandle {
    app_data_db: AppDataDb,
    icons_db: IconsDb,
}
