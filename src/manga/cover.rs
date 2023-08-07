use anyhow::{Context, Result};
use mangadex_api::MangaDexClient;
use mangadex_api_schema_rust::{v5::CoverAttributes, ApiObject};
use std::fs::{create_dir_all, File};
use std::io::Write;
use std::path::PathBuf;
use uuid::Uuid;

#[derive(Debug, Clone)]
pub struct Cover {
    pub id: Uuid,
    pub volume: Option<u32>,
    pub sub_volume: Option<u32>,
    pub path: Option<PathBuf>,
}

impl Cover {
    #[allow(dead_code)]
    pub fn new(
        id: Uuid,
        volume: Option<u32>,
        sub_volume: Option<u32>,
        path: Option<PathBuf>,
    ) -> Self {
        Self {
            id,
            volume,
            sub_volume,
            path,
        }
    }

    pub async fn download(&self, index: usize, client: &MangaDexClient) -> Result<()> {
        if let Some(path) = self.path.clone() {
            create_dir_all(&path)?;
            if let (original_filename, Some(bytes)) = client
                .download()
                .cover()
                .build()?
                .via_cover_id(self.id)
                .await?
            {
                let filename = PathBuf::from(original_filename);
                let mut result = filename.to_owned();
                result.set_file_name(format!("cover{}", index));
                if let Some(ext) = filename.extension() {
                    result.set_extension(ext);
                }
                let mut file = File::create(path.join(result))?;
                file.write_all(&bytes)?;
                Ok(())
            } else {
                anyhow::bail!("Missing Bytes for Cover!");
            }
        } else {
            anyhow::bail!("Missing Cover Path!");
        }
    }
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

        let sub_volume = volume_n.and_then(|volume| {
            if volume.fract() > 0.0 {
                Some((volume.fract() * 10.0) as u32)
            } else {
                None
            }
        });

        Ok(Cover {
            id: value.id,
            path: None,
            volume: volume_n.map(|volume| volume as u32),
            sub_volume,
        })
    }
}
