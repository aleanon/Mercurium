pub const CREATE_TABLE_NFT_IMAGES: &'static str = "CREATE TABLE IF NOT EXISTS
    nft_images (
        nfid TEXT NOT NULL PRIMARY KEY,
        image_data BLOB NOT NULL,
        resource_address BLOB NOT NULL,
        FOREIGN KEY(resource_address) REFERENCES resource_images(resource_address)
    )
";

pub const UPSERT_NFT_IMAGE: &'static str = "INSERT INTO
    nft_images (
        nfid,
        image_data,
        resource_address
    )
    VALUES (?,?,?)
    ON CONFLICT (nfid)
    DO UPDATE SET
        image_data = excluded.image_data
";

