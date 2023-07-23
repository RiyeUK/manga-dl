use anyhow::{Context, Result};
use mangadex_api_schema_rust::{v5::CoverAttributes, ApiObject};
use std::path::PathBuf;
use uuid::Uuid;

#[derive(Debug)]
pub struct Cover {
    pub id: Uuid,
    pub volume: Option<u32>,
    pub sub_volume: Option<u32>,
    pub path: Option<PathBuf>,
}

impl<T> TryFrom<ApiObject<CoverAttributes, T>> for Cover {
    type Error = anyhow::Error;

    fn try_from(value: ApiObject<CoverAttributes, T>) -> Result<Self, Self::Error> {
        let volume_n: Option<f64> = value.attributes.volume.and_then(|volume| {
            volume
                .parse::<f64>()
                .with_context(|| {
                    format!(
                        "Unable to parse {:?} as a chapter number for chapter {}.",
                        volume, &value.id
                    )
                })
                .ok()
        });

        let sub_volume = volume_n
            .map(|volume| {
                if volume.fract() > 0.0 {
                    Some((volume.fract() * 10.0) as u32)
                } else {
                    None
                }
            })
            .flatten();

        Ok(Cover {
            id: value.id,
            path: None,
            volume: volume_n.map(|volume| volume as u32),
            sub_volume,
        })
    }
}

// impl Cover {

// }
