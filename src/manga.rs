#[allow(unused_imports)]
use anyhow::{Context, Result};
#[allow(unused_imports)]
use indicatif::{MultiProgress, ProgressBar, ProgressStyle};
use mangadex_api::MangaDexClient;
use std::path::PathBuf;
use uuid::Uuid;

pub mod get;

mod chapter;
mod cover;
mod mangadata;
mod volume;

use mangadata::MangaData;
use volume::Volume;

#[derive(Debug)]
pub struct Manga {
    pub client: MangaDexClient,
    pub id: Uuid,
    pub metadata: MangaData,
    pub volumes: Vec<Volume>,
    pub path: PathBuf,
}

impl Manga {
    pub async fn download(&self) -> Result<()> {
        println!("START DOWNLOAD!");
        let mp = MultiProgress::new();
        let style = ProgressStyle::with_template(
            "[{elapsed_precise}] {bar:40.cyan/blue} {pos:>7}/{len:7} {msg}",
        )?;
        let vol_bar = mp.add(
            ProgressBar::new(self.volumes.len().try_into()?)
                .with_message("Downloading Volumes")
                .with_style(style),
        );
        for volume in self.volumes.iter() {
            vol_bar.set_message(format!("Vol. {:?}", volume.volume));
            vol_bar.inc(1);
            volume.download(&mp, &self.client).await?;
        }
        vol_bar.finish_with_message("Downloaded Volumes");

        Ok(())
    }
}
