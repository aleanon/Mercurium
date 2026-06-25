pub const CREATE_TABLE_RESOURCE_IMAGES: &'static str = "CREATE TABLE IF NOT EXISTS
    resource_images (
        resource_address BLOB NOT NULL PRIMARY KEY,
        image_data BLOB NOT NULL
    )
";

pub const UPSERT_RESOURCE_IMAGE: &'static str = "INSERT INTO
    resource_images (
        resource_address,
        image_data
    )
    VALUES (?,?)
    ON CONFLICT (resource_address)
    DO UPDATE SET
        image_data = excluded.image_data
";
