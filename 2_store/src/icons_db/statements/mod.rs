use deps_two::*;

pub mod nft_images;
pub mod resource_images;

use self::{nft_images::CREATE_TABLE_NFT_IMAGES, resource_images::CREATE_TABLE_RESOURCE_IMAGES};

pub const CREATE_ALL_ICONCACHE_TABLES_BATCH: &'static str = const_format::formatcp!(
    "BEGIN;
    {CREATE_TABLE_RESOURCE_IMAGES};
    {CREATE_TABLE_NFT_IMAGES};
    COMMIT;"
);

#[cfg(test)]
mod test {

    use crate::database::test::{execute_batch_stmt, execute_stmt};

    use super::*;

    #[test]
    fn test_create_all_tables_iconcache() {
        let result = execute_batch_stmt(CREATE_ALL_ICONCACHE_TABLES_BATCH);
        println!("{:?}", result);
        assert!(result.is_ok());
    }

    #[test]
    fn test_create_table_resource_images() {
        let result = execute_stmt(CREATE_TABLE_RESOURCE_IMAGES);
        println!("{:?}", result);
        assert!(result.is_ok());
    }

    #[test]
    fn test_create_table_nft_images() {
        let result = execute_stmt(CREATE_TABLE_NFT_IMAGES);
        println!("{:?}", result);
        assert!(result.is_ok());
    }
}
