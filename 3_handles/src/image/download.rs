use deps::*;

use std::{
    collections::{BTreeMap, HashMap},
    io::{BufWriter, Cursor},
};

use bytes::Bytes;
use debug_print::debug_println;
use futures::future::join_all;
// use iced::{futures::future::join_all, widget::image::Handle};
use image::DynamicImage;
use store::IconsDb;
use types::{address::ResourceAddress, Network};

use crate::image::resize::{resize_small_dimensions, resize_standard_dimensions};

// pub async fn download_resize_and_store_resource_icons_as_handle(
//     icon_urls: BTreeMap<ResourceAddress, String>,
//     network: Network,
// ) -> HashMap<ResourceAddress, Handle> {
//     let icon_cache = IconsDb::get(network);
//     let tasks = icon_urls.into_iter().map(|(resource_address, url)| {
//         tokio::spawn(async move {
//             download_image(&url).await.and_then(|image| {
//                 let mut encoded_standard = BufWriter::new(Cursor::new(Vec::new()));
//                 resize_standard_dimensions(&image)
//                     .write_to(&mut encoded_standard, image::ImageFormat::Png)
//                     .ok()?;

//                 let mut encoded_small = BufWriter::new(Cursor::new(Vec::new()));
//                 resize_small_dimensions(&image)
//                     .write_to(&mut encoded_small, image::ImageFormat::Png)
//                     .ok()?;

//                 Some((
//                     resource_address,
//                     encoded_standard.into_inner().ok()?.into_inner(),
//                     encoded_small.into_inner().ok()?.into_inner(),
//                 ))
//             })
//         })
//     });

//     let (icons, icons_data) = join_all(tasks)
//         .await
//         .into_iter()
//         .filter_map(|join_result| {
//             join_result
//                 .ok()?
//                 .and_then(|(resource_address, image_standard, image_small)| {
//                     let handle = iced::widget::image::Handle::from_bytes(image_small);
//                     Some((
//                         (resource_address.clone(), handle),
//                         (resource_address, image_standard),
//                     ))
//                 })
//         })
//         .fold(
//             (HashMap::new(), HashMap::new()),
//             |(mut icon_handles_acc, mut icons_data_acc), (icon_handles, icons_data)| {
//                 icon_handles_acc.insert(icon_handles.0, icon_handles.1);
//                 icons_data_acc.insert(icons_data.0, icons_data.1);
//                 (icon_handles_acc, icons_data_acc)
//             },
//         );

//     if let Some(icon_cache) = icon_cache {
//         icon_cache.upsert_resource_icons(icons_data).await.ok();
//     } else {
//         debug_println!("Icon cache not found")
//     }

//     icons
// }

async fn download_image(url: &String) -> Option<DynamicImage> {
    let response = reqwest::get(url).await.ok()?;

    let bytes = response.bytes().await.ok()?;
    let reader = image::ImageReader::new(Cursor::new(&bytes));
    let with_guessed_format = reader.with_guessed_format().ok()?;
    with_guessed_format.decode().ok()
}

pub async fn download_resize_and_store_resource_icons(
    icon_urls: BTreeMap<ResourceAddress, String>,
    network: Network,
) -> HashMap<ResourceAddress, Bytes> {
    let icon_cache = IconsDb::get(network);
    let tasks = icon_urls.into_iter().map(|(resource_address, url)| {
        tokio::spawn(async move {
            download_image(&url).await.and_then(|image| {
                let mut encoded_standard = BufWriter::new(Cursor::new(Vec::new()));
                resize_standard_dimensions(&image)
                    .write_to(&mut encoded_standard, image::ImageFormat::Png)
                    .ok()?;

                let mut encoded_small = BufWriter::new(Cursor::new(Vec::new()));
                resize_small_dimensions(&image)
                    .write_to(&mut encoded_small, image::ImageFormat::Png)
                    .ok()?;

                Some((
                    resource_address,
                    encoded_standard.into_inner().ok()?.into_inner(),
                    encoded_small.into_inner().ok()?.into_inner(),
                ))
            })
        })
    });

    let (icons, icons_data) = join_all(tasks)
        .await
        .into_iter()
        .filter_map(|join_result| {
            join_result
                .ok()?
                .and_then(|(resource_address, image_standard, image_small)| {
                    let bytes = Bytes::from_owner(image_small);
                    Some((
                        (resource_address.clone(), bytes),
                        (resource_address, image_standard),
                    ))
                })
        })
        .fold(
            (HashMap::new(), HashMap::new()),
            |(mut icon_handles_acc, mut icons_data_acc), (icon_handles, icons_data)| {
                icon_handles_acc.insert(icon_handles.0, icon_handles.1);
                icons_data_acc.insert(icons_data.0, icons_data.1);
                (icon_handles_acc, icons_data_acc)
            },
        );

    if let Some(icon_cache) = icon_cache {
        icon_cache.upsert_resource_icons(icons_data).await.ok();
    } else {
        debug_println!("Icon cache not found")
    }

    icons
}