use std::collections::{BTreeMap, HashMap};

use super::{
    statements::{nft_images, resource_images},
    DbError, IconsDb,
};

use async_sqlite::rusqlite::params;
use types::address::{Address, ResourceAddress};

impl IconsDb {
    pub async fn upsert_resource_icons(
        &self,
        icons: HashMap<ResourceAddress, Vec<u8>>,
    ) -> Result<(), DbError> {
        self.transaction(resource_images::UPSERT_RESOURCE_IMAGE, |cached_stmt| {
            for (resource_address, image_data) in icons {
                cached_stmt.execute(params![resource_address, image_data])?;
            }
            Ok(())
        })
        .await
    }

    pub async fn upsert_resource_icon(
        &self,
        resource_address: ResourceAddress,
        image_data: Vec<u8>,
    ) -> Result<(), DbError> {
        self.conn(move |conn| {
            conn.execute(
                resource_images::UPSERT_RESOURCE_IMAGE,
                params![resource_address, image_data],
            )
        })
        .await
        .map(|_| ())
    }

    pub async fn upsert_nft_images(
        &self,
        resource_address: ResourceAddress,
        images: BTreeMap<String, Vec<u8>>,
    ) -> Result<(), DbError> {
        self.transaction(nft_images::UPSERT_NFT_IMAGE, move |cached_stmt| {
            for (mut nfid, image_data) in images {
                nfid.push_str(resource_address.checksum_as_str());
                cached_stmt.execute(params![nfid, image_data, resource_address])?;
            }
            Ok(())
        })
        .await
    }

    pub async fn upsert_nft_image(
        &self,
        resource_address: ResourceAddress,
        mut nfid: String,
        image_data: Vec<u8>,
    ) -> Result<(), DbError> {
        self.conn(move |conn| {
            nfid.push_str(resource_address.as_str());
            conn.execute(
                nft_images::UPSERT_NFT_IMAGE,
                params![nfid, image_data, resource_address],
            )
        })
        .await
        .map(|_| ())
    }
}
