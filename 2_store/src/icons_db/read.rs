use deps::*;

use crate::{DbError, IconsDb};

use async_sqlite::rusqlite::{self, Row};
use std::collections::BTreeMap;
use types::address::{Address, ResourceAddress};

impl IconsDb {
    // pub async fn get_all_resource_icons<T>(&self) -> Result<T, DbError>
    // where
    //     T: FromIterator<(ResourceAddress, Vec<u8>)> + Send + 'static,
    // {
    //     self.conn(|conn| {
    //         conn.prepare_cached("SELECT * FROM resource_images")?
    //             .query_map([], |row| {
    //                 let resource_address: ResourceAddress = row.get(0)?;
    //                 let image_data: Vec<u8> = row.get(1)?;
    //                 Ok((resource_address, image_data))
    //             })?
    //             .collect()
    //     })
    //     .await
    // }

    pub async fn get_all_resource_icons<T>(&self) -> Result<T, DbError>
    where
        T: FromIterator<(ResourceAddress, Vec<u8>)> + Send + 'static,
    {
        self.query_map(
            "SELECT * FROM resource_images",
            [], 
            Self::get_resource_address_and_image_data_from_row
        ).await
    }

    // pub async fn get_resource_icon(
    //     &self,
    //     resource_address: ResourceAddress,
    // ) -> Result<(ResourceAddress, Vec<u8>), DbError> {
    //     self.conn(move |conn| {
    //         conn.query_row(
    //             "SELECT * FROM resource_images WHERE resource_address = ?",
    //             [resource_address],
    //             |row| {
    //                 let resource_address: ResourceAddress = row.get(0)?;
    //                 let image_data: Vec<u8> = row.get(1)?;
    //                 Ok((resource_address, image_data))
    //             },
    //         )
    //     })
    //     .await
    // }

    pub async fn get_resource_icon(
        &self,
        resource_address: ResourceAddress,
    ) -> Result<(ResourceAddress, Vec<u8>), DbError> {
        self.query_row(
            "SELECT * FROM resource_images WHERE resource_address = ?",
            [resource_address],
            Self::get_resource_address_and_image_data_from_row
        )
        .await
    }

    // pub async fn get_all_nft_images_for_resource(
    //     &self,
    //     resource_address: ResourceAddress,
    // ) -> Result<(ResourceAddress, BTreeMap<String, Vec<u8>>), DbError> {
    //     let resource_address_params = resource_address.clone();
    //     let btree_map = self
    //         .conn(|conn| {
    //             conn.prepare_cached("SELECT * FROM nft_images WHERE resource_address = ?")?
    //                 .query_map([resource_address_params], |row| {
    //                     let mut nfid: String = row.get(0)?;
    //                     let _ = nfid.split_off(nfid.len() - ResourceAddress::CHECKSUM_LENGTH);
    //                     let image_data: Vec<u8> = row.get(1)?;
    //                     Ok((nfid, image_data))
    //                 })?
    //                 .collect::<Result<BTreeMap<String, Vec<u8>>, rusqlite::Error>>()
    //         })
    //         .await?;
    //     Ok((resource_address, btree_map))
    // }

    pub async fn get_all_nft_images_for_resource(
        &self,
        resource_address: ResourceAddress,
    ) -> Result<(ResourceAddress, BTreeMap<String, Vec<u8>>), DbError> {
        let resource_address_params = resource_address.clone();
        let btree_map = self.query_map(
            "SELECT * FROM nft_images WHERE resource_address = ?", 
            [resource_address_params],
            Self::get_nfid_and_image_data_from_row
        )
        .await?;

        Ok((resource_address, btree_map))
    }
    
    // pub async fn get_nft_image(
    //     &self,
    //     resource_address: ResourceAddress,
    //     nfid: String,
    // ) -> Result<(ResourceAddress, String, Vec<u8>), DbError> {
    //     self.conn(move |conn| {
    //         let mut nfid_param = nfid.clone();
    //         nfid_param.push_str(resource_address.checksum_as_str());

    //         conn.query_row(
    //             "SELECT * FROM nft_images WHERE nfid =?",
    //             [nfid_param],
    //             |row| {
    //                 let image_data: Vec<u8> = row.get(1)?;
    //                 Ok((resource_address, nfid, image_data))
    //             },
    //         )
    //     })
    //     .await
    // }

    pub async fn get_nft_image(
        &self,
        resource_address: ResourceAddress,
        nfid: String,
    ) -> Result<(ResourceAddress, String, Vec<u8>), DbError> {
        let mut nfid_param = nfid.clone();
        nfid_param.push_str(resource_address.checksum_as_str());

        self.query_row(
            "SELECT * FROM nft_images WHERE nfid =?",
            [nfid_param],
            Self::get_nfid_and_image_data_from_row
        )
        .await
        .map(|(nfid, image)| (resource_address, nfid, image))
    }

    fn get_resource_address_and_image_data_from_row(row: &Row<'_>) -> Result<(ResourceAddress, Vec<u8>), rusqlite::Error> {
        let resource_address: ResourceAddress = row.get(0)?;
        let image_data: Vec<u8> = row.get(1)?;
        Ok((resource_address, image_data))
    }

    fn get_nfid_and_image_data_from_row(row: &Row<'_>) -> Result<(String, Vec<u8>), rusqlite::Error> {
        let mut nfid: String = row.get(0)?;
        let _ = nfid.split_off(nfid.len() - ResourceAddress::CHECKSUM_LENGTH);
        let image_data: Vec<u8> = row.get(1)?;
        Ok((nfid, image_data))
    }
}
