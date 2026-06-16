//! HTTP-backed [`IconProvider`] adapter: download icon bytes over HTTP and resize them.
//! Moved here from the former `handles::image` module.

use deps::*;

use std::collections::{BTreeMap, HashMap};
use std::io::Cursor;

use async_trait::async_trait;
use futures::future::join_all;
use image::DynamicImage;
use types::address::ResourceAddress;

use crate::port::IconProvider;
use crate::resize::{resize_small_dimensions, resize_standard_dimensions};

/// The default HTTP icon provider (stateless).
#[derive(Debug, Clone, Copy, Default)]
pub struct HttpIconProvider;

#[async_trait]
impl IconProvider for HttpIconProvider {
    async fn fetch_icon(&self, url: &str) -> Option<Vec<u8>> {
        let image = download_image(url).await?;
        resize_standard_dimensions(&image)
    }

    async fn fetch_icons(
        &self,
        urls: BTreeMap<ResourceAddress, String>,
    ) -> HashMap<ResourceAddress, (Vec<u8>, Vec<u8>)> {
        let tasks = urls.into_iter().map(|(resource_address, url)| {
            tokio::spawn(async move {
                download_image(&url).await.and_then(|image| {
                    let standard = resize_standard_dimensions(&image)?;
                    let small = resize_small_dimensions(&image)?;
                    Some((resource_address, (small, standard)))
                })
            })
        });

        join_all(tasks)
            .await
            .into_iter()
            .filter_map(|join_result| join_result.ok()?)
            .collect()
    }
}

async fn download_image(url: &str) -> Option<DynamicImage> {
    let response = reqwest::get(url).await.ok()?;
    let bytes = response.bytes().await.ok()?;
    let reader = image::ImageReader::new(Cursor::new(&bytes));
    reader.with_guessed_format().ok()?.decode().ok()
}
