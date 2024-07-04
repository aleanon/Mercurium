use std::ops::Deref;
use std::ops::DerefMut;

use bytes::Bytes;
use iced::widget::image::Handle;
use image::DynamicImage;
use image::EncodableLayout;
use serde::{Deserialize, Serialize};

///Stores the raw image data that is saved in the database, also creates a Handle for fast image processing
#[derive(Debug, Clone)]
pub struct Icon {
    data: Bytes,
    handle: Handle,
}

impl Icon {
    pub fn new(data: Bytes) -> Self {
        // if let Ok(image) = image::load_from_memory(data.as_bytes()) {
        //     let resized = image.resize(40, 40, image::imageops::FilterType::CatmullRom);
        //     data = Bytes::from(resized.as_bytes().to_vec());
        // }

        let handle = Handle::from_memory(data.clone());
        Self { data, handle }
    }

    pub fn new_with_handle(data: Bytes, handle: Handle) -> Self {
        Self { data, handle }
    }
    pub fn from_image(image: &DynamicImage) -> Self {
        let handle = Handle::from_memory(image.as_bytes().to_vec());
        Self {
            data: Bytes::from(image.as_bytes().to_vec()),
            handle,
        }
    }

    pub fn handle(&self) -> Handle {
        self.handle.clone()
    }

    pub fn bytes(&self) -> &[u8] {
        &self.data.as_bytes()
    }
}

impl From<Bytes> for Icon {
    fn from(data: Bytes) -> Self {
        let handle = Handle::from_memory(data.clone());
        Self { data, handle }
    }
}

impl From<Vec<u8>> for Icon {
    fn from(value: Vec<u8>) -> Self {
        let data = Bytes::from(value);
        let handle = Handle::from_memory(data.clone());
        Self { data, handle }
    }
}

impl From<&[u8]> for Icon {
    fn from(value: &[u8]) -> Self {
        let data = Bytes::from(value.to_vec());
        let handle = Handle::from_memory(data.clone());
        Self { data, handle }
    }
}

impl Deref for Icon {
    type Target = Bytes;
    fn deref(&self) -> &Self::Target {
        &self.data
    }
}

impl DerefMut for Icon {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.data
    }
}

impl Into<Bytes> for Icon {
    fn into(self) -> Bytes {
        self.data
    }
}

impl Into<Vec<u8>> for Icon {
    fn into(self) -> Vec<u8> {
        self.data.to_vec()
    }
}

impl Serialize for Icon {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_newtype_struct("Icon", self.data.as_ref())
    }
}

impl<'de> Deserialize<'de> for Icon {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        use serde::de::Visitor;
        use std::fmt;

        struct IconVisitor;

        impl<'de> Visitor<'de> for IconVisitor {
            type Value = Icon;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str("struct Icon")
            }

            fn visit_bytes<E>(self, v: &[u8]) -> Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                Ok(Icon::new(v.to_vec().into()))
            }
        }
        deserializer.deserialize_newtype_struct("Icon", IconVisitor)
    }
}

impl rusqlite::types::FromSql for Icon {
    fn column_result(value: rusqlite::types::ValueRef<'_>) -> rusqlite::types::FromSqlResult<Self> {
        match value {
            rusqlite::types::ValueRef::Blob(bytes) => Ok(bytes.to_vec().into()),
            _ => Err(rusqlite::types::FromSqlError::InvalidType),
        }
    }
}

impl rusqlite::types::ToSql for Icon {
    fn to_sql(&self) -> rusqlite::Result<rusqlite::types::ToSqlOutput<'_>> {
        Ok(rusqlite::types::ToSqlOutput::Owned(
            rusqlite::types::Value::Blob(self.to_vec()),
        ))
    }
}
