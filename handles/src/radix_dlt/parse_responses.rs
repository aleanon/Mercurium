use std::collections::HashMap;
use std::io::{BufWriter, Cursor};
use std::num::NonZeroU32;
use std::path::PathBuf;
use std::str::FromStr;
use std::sync::Arc;

use bytes::Bytes;
use image::imageops::FilterType;
use types::non_fungibles::NonFungible;
use types::response_models::{Entity, FungibleResource, NonFungibleResource};
use types::{AppPath, Fungible, Icon, MetaData, NFIDs, RadixDecimal, ResourceAddress};

use crate::filesystem::resize_image::resize_image;

async fn parse_fungible_response(
    fungible_resources: Arc<HashMap<String, FungibleResource>>,
    fungible: Entity,
) -> Option<Fungible> {
    let (last_updated, amount) = match fungible_resources.get(&*fungible.address) {
        Some(fungible_resource) => {
            let mut amount = RadixDecimal::ZERO;
            let mut last_updated = 0;
            for vault in &fungible_resource.vaults.items {
                amount +=
                    RadixDecimal::from_str(&vault.amount).unwrap_or_else(|_| RadixDecimal::ZERO);
                if last_updated < vault.last_updated_at_state_version {
                    last_updated = vault.last_updated_at_state_version
                }
            }
            (last_updated, amount.into())
        }
        None => (0, RadixDecimal::ZERO.into()),
    };

    let address = ResourceAddress::from_str(&fungible.address).ok()?;

    let mut name = None;
    let mut symbol = None;
    let mut description = None;
    let mut icon_url = None;
    let mut metadata = MetaData::new();
    let total_supply = fungible.details.total_supply.unwrap_or(String::new());

    for item in fungible.metadata.items {
        match &*item.key {
            "name" => name = item.value.typed.value,
            "symbol" => symbol = item.value.typed.value,
            "description" => description = item.value.typed.value,
            "icon_url" => icon_url = item.value.typed.value.filter(|value| value.len() != 0),
            _ => metadata.push(item.into()),
        }
    }

    let icon = get_icon(icon_url, &address).await;

    let fungible = Fungible {
        address,
        amount,
        total_supply,
        description,
        name: name.unwrap_or(String::new()),
        symbol: symbol.unwrap_or(String::new()),
        icon,
        last_updated_at_state_version: last_updated as i64,
        metadata,
    };
    Some(fungible)
}

/// Takes a map where the address is the key of the map, the `NonFungibleResource` is the response
/// model from for a NonFungible after a gateway request
async fn parse_non_fungible_response(
    non_fungible_resources: Arc<HashMap<String, NonFungibleResource>>,
    non_fungible: Entity,
) -> Option<NonFungible> {
    let (last_updated, nfids) = match non_fungible_resources.get(&*non_fungible.address) {
        Some(non_fungible_resource) => {
            let mut last_updated = 0;
            for vault in &non_fungible_resource.vaults.items {
                if last_updated < vault.last_updated_at_state_version {
                    last_updated = vault.last_updated_at_state_version
                }
            }
            let nfids = NFIDs::from(&non_fungible_resource.vaults);
            (last_updated, nfids)
        }
        None => (0, NFIDs::new()),
    };

    let address = ResourceAddress::from_str(&non_fungible.address).ok()?;

    let mut name = None;
    let mut symbol = None;
    let mut description = None;
    let mut icon_url = None;
    let mut metadata = MetaData::new();
    let _current_supply = non_fungible.details.total_supply.unwrap_or(String::new());

    for item in non_fungible.metadata.items {
        match &*item.key {
            "name" => name = item.value.typed.value,
            "symbol" => symbol = item.value.typed.value,
            "description" => description = item.value.typed.value,
            "icon_url" => icon_url = item.value.typed.value.filter(|value| value.len() != 0),
            _ => metadata.push(item.into()),
        }
    }

    let icon = get_icon(icon_url, &address).await;

    let non_fungible = NonFungible {
        address,
        description,
        name: name.unwrap_or(String::new()),
        symbol: symbol.unwrap_or(String::new()),
        icon,
        nfids,
        last_updated_at_state_version: last_updated as i64,
        metadata,
    };
    Some(non_fungible)
}

async fn get_icon(icon_url: Option<String>, resource_address: &ResourceAddress) -> Option<Icon> {
    let url = icon_url?;
    if let Ok(app_path) = AppPath::new() {
        let mut icon_path = app_path.icons_directory().clone();
        icon_path.push(resource_address.as_str());
        if icon_path.exists() {
            if let Ok(image) = image::open(&icon_path) {
                if let Some(resized) = resize_image(
                    &image,
                    NonZeroU32::new(50).unwrap(),
                    NonZeroU32::new(50).unwrap(),
                ) {
                    Some(Icon::new(Bytes::from(resized.buffer().to_vec())))
                } else
                //Could not resize image
                {
                    download_icon(&url, Some(&mut icon_path)).await
                }
            } else
            //Could not open image
            {
                download_icon(&url, Some(&mut icon_path)).await
            }
        } else
        //Icon path does not exist
        {
            download_icon(&url, Some(&mut icon_path)).await
        }
    } else
    //Unable to determine icons directory
    {
        download_icon(&url, None).await
    }
}

async fn download_icon(url: &String, icon_path: Option<&mut PathBuf>) -> Option<Icon> {
    let response = reqwest::get(url).await.ok()?;

    let bytes = response.bytes().await.ok()?;
    let reader = image::io::Reader::new(Cursor::new(&bytes));
    let with_guessed_format = reader.with_guessed_format().ok()?;
    let format = with_guessed_format.format()?;
    let image = with_guessed_format.decode().ok()?;

    if let Some(path) = icon_path {
        path.set_extension(crate::filesystem::image_extension::get_extension(&format));
        image.save_with_format(path, format).unwrap_or(());
    }

    let resized = image.resize(50, 50, FilterType::Lanczos3);
    let mut write_buffer = BufWriter::new(Cursor::new(Vec::new()));
    resized.write_to(&mut write_buffer, format).ok()?;

    let inner = write_buffer.into_inner().ok()?.into_inner();
    let icon = Icon::new(Bytes::from(inner));

    Some(icon)
}
