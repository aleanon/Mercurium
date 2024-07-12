use std::collections::{BTreeMap, HashMap};

use rusqlite::{params, OpenFlags};
use tokio_rusqlite::Connection;
use types::{
    app_path::AppPath, Network, ResourceAddress,
};

use crate::statements;

#[derive(Debug)]
pub struct IconCache {
    connection: Connection,
}

impl IconCache {
    pub async fn load(network: Network) -> Result<Self, tokio_rusqlite::Error> {
        let path = AppPath::get().icon_cache_ref(network);
        let connection = Connection::open_with_flags(
            path,
            OpenFlags::SQLITE_OPEN_READ_WRITE | OpenFlags::SQLITE_OPEN_CREATE,
        )
        .await?;
        let iconcache = Self {
            connection: connection,
        };

        iconcache.create_tables_if_not_exist().await?;

        Ok(iconcache)
    }

    pub async fn create_tables_if_not_exist(&self) -> Result<(), tokio_rusqlite::Error> {
        self.connection
            .call_unwrap(|conn| {
                conn.execute(statements::create::CREATE_TABLE_RESOURCE_IMAGES, [])?;
                conn.execute(statements::create::CREATE_TABLE_NFT_IMAGES, [])?;
                Ok(())
            })
            .await
    }

    pub async fn get_all_resource_icons(
        &self,
    ) -> Result<HashMap<ResourceAddress, Vec<u8>>, tokio_rusqlite::Error> {
        let result = self
            .connection
            .call_unwrap(|conn| {
                conn.prepare_cached("SELECT * FROM resource_images")?
                    .query_map([], |row| {
                        let resource_address: ResourceAddress = row.get(0)?;
                        let image_data: Vec<u8> = row.get(1)?;
                        Ok((resource_address, image_data))
                    })?
                    .collect::<Result<HashMap<ResourceAddress, Vec<u8>>, rusqlite::Error>>()
            })
            .await?;
        Ok(result)
    }

    pub async fn get_resource_icon(
        &self,
        resource_address: ResourceAddress,
    ) -> Result<(ResourceAddress, Vec<u8>), tokio_rusqlite::Error> {
        Ok(self
            .connection
            .call_unwrap(move |conn| {
                conn.query_row(
                    "SELECT * FROM resource_images WHERE resource_address = ?",
                    [resource_address],
                    |row| {
                        let resource_address: ResourceAddress = row.get(0)?;
                        let image_data: Vec<u8> = row.get(1)?;
                        Ok((resource_address, image_data))
                    },
                )
            })
            .await?)
    }

    pub async fn get_all_nft_images_for_resource(
        &self,
        resource_address: ResourceAddress,
    ) -> Result<(ResourceAddress, BTreeMap<String, Vec<u8>>), tokio_rusqlite::Error> {
        let resource_address_params = resource_address.clone();
        let btree_map = self
            .connection
            .call_unwrap(|conn| {
                conn.prepare_cached("SELECT * FROM nft_images WHERE resource_address = ?")?
                    .query_map([resource_address_params], |row| {
                        let mut nfid: String = row.get(0)?;
                        let _ = nfid.split_off(nfid.len() - ResourceAddress::CHECKSUM_LEN);
                        let image_data: Vec<u8> = row.get(1)?;
                        Ok((nfid, image_data))
                    })?
                    .collect::<Result<BTreeMap<String, Vec<u8>>, rusqlite::Error>>()
            })
            .await?;
        Ok((resource_address, btree_map))
    }

    pub async fn get_nft_image(
        &self,
        resource_address: ResourceAddress,
        nfid: String,
    ) -> Result<(ResourceAddress, String, Vec<u8>), tokio_rusqlite::Error> {
        Ok(self
            .connection
            .call_unwrap(move |conn| {
                let mut nfid_param = nfid.clone();
                nfid_param.push_str(resource_address.checksum_str());

                conn.query_row(
                    "SELECT * FROM nft_images WHERE nfid =?",
                    [nfid_param],
                    |row| {
                        let image_data: Vec<u8> = row.get(1)?;
                        Ok((resource_address, nfid, image_data))
                    },
                )
            })
            .await?)
    }

    pub async fn upsert_resource_icons(
        &self,
        icons: HashMap<ResourceAddress, Vec<u8>>,
    ) -> Result<(), tokio_rusqlite::Error> {
        self.connection
            .call_unwrap(move |conn| {
                let tx = conn.transaction()?;

                {
                    let mut stmt = tx.prepare_cached(statements::upsert::UPSERT_RESOURCE_IMAGE)?;

                    for (resource_address, image_data) in icons {
                        stmt.execute(params![resource_address, image_data])?;
                    }
                }

                tx.commit()
            })
            .await?;
        Ok(())
    }

    pub async fn upsert_resource_icon(
        &self,
        resource_address: ResourceAddress,
        image_data: Vec<u8>,
    ) -> Result<(), tokio_rusqlite::Error> {
        self.connection
            .call_unwrap(move |conn| {
                conn.execute(
                    statements::upsert::UPSERT_RESOURCE_IMAGE,
                    params![resource_address, image_data],
                )
            })
            .await?;
        Ok(())
    }

    pub async fn upsert_nft_images(
        &self,
        resource_address: ResourceAddress,
        images: BTreeMap<String, Vec<u8>>,
    ) -> Result<(), tokio_rusqlite::Error> {
        self.connection
            .call_unwrap(move |conn| {
                let tx = conn.transaction()?;

                {
                    let mut stmt = tx.prepare_cached(statements::upsert::UPSERT_NFT_IMAGE)?;

                    for (mut nfid, image_data) in images {
                        nfid.push_str(resource_address.checksum_str());
                        stmt.execute(params![nfid, image_data, resource_address])?;
                    }
                }

                tx.commit()
            })
            .await?;
        Ok(())
    }

    pub async fn upsert_nft_image(
        &self,
        resource_address: ResourceAddress,
        mut nfid: String,
        image_data: Vec<u8>,
    ) -> Result<(), tokio_rusqlite::Error> {
        self.connection
            .call_unwrap(move |conn| {
                nfid.push_str(resource_address.as_str());
                conn.execute(
                    statements::upsert::UPSERT_NFT_IMAGE,
                    params![nfid, image_data, resource_address],
                )
            })
            .await?;
        Ok(())
    }
}
