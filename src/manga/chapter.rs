use anyhow::{Context, Result};
use futures::{stream, StreamExt};
use mangadex_api::{
    utils::download::{chapter::DownloadMode, DownloadElement},
    MangaDexClient,
};
use mangadex_api_schema_rust::{v5::ChapterAttributes, ApiObject};
use std::path::PathBuf;
use uuid::Uuid;

#[derive(Debug)]
pub struct Chapter {
    pub id: Uuid,
    pub title: Option<String>,
    pub volume: Option<u32>,
    pub chapter: u32,
    pub sub_chapter: Option<u32>,
    pub path: Option<PathBuf>,
    pub pages: u32,
}

impl<T> TryFrom<ApiObject<ChapterAttributes, T>> for Chapter {
    type Error = anyhow::Error;

    fn try_from(value: ApiObject<ChapterAttributes, T>) -> Result<Self, Self::Error> {
        let chapter_n: f64 = match value.attributes.chapter {
            Some(ch) => ch.parse().with_context(|| {
                format!(
                    "Unable to parse {:?} as a chapter number for chapter {}.",
                    ch, &value.id
                )
            })?,
            None => anyhow::bail!("No Chapter number for {}", &value.id),
        };

        let volume: Option<u32> = match value.attributes.volume {
            Some(vol) => Some(vol.parse().with_context(|| {
                format!(
                    "Unable to parse {:?} as a volume number in chapter {}",
                    vol, &value.id
                )
            })?),
            None => None,
        };

        let sub_chapter: Option<u32> = {
            if chapter_n.fract() > 0.0 {
                Some((chapter_n.fract() * 10.0) as u32)
            } else {
                None
            }
        };

        Ok(Chapter {
            id: value.id,
            title: Some(value.attributes.title),
            volume,
            chapter: chapter_n as u32,
            sub_chapter,
            path: None,
            pages: value.attributes.pages,
        })
    }
}

impl Chapter {
    async fn save_page(data: DownloadElement, index: usize, length: usize) -> anyhow::Result<()> {
        println!("Saved Page {}/{}", index, length);
        Ok(())
    }

    pub async fn download_stream(&self, client: MangaDexClient) -> Result<Vec<anyhow::Result<()>>> {
        let file_names = client
            .download()
            .chapter(self.id)
            .mode(DownloadMode::Normal)
            .report(true)
            .build()?
            .build_at_home_urls()
            .await?;

        let len = file_names.len();
        let results = stream::iter(file_names)
            .enumerate()
            .map(|(index, filename)| async move {
                let data = filename.download().await?;
                Chapter::save_page(data, index + 1, len).await?;
                Ok(())
            })
            .buffer_unordered(5)
            .collect::<Vec<_>>()
            .await;

        Ok(results)
    }
}
