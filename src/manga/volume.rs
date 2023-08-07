use super::{chapter::Chapter, cover::Cover};
use anyhow::{Context, Result};
use indicatif::{MultiProgress, ProgressBar, ProgressStyle};
use mangadex_api::MangaDexClient;
use std::path::PathBuf;

#[derive(Debug)]
pub struct Volume {
    pub covers: Vec<Cover>,
    pub volume: Option<u32>,
    pub chapters: Vec<Chapter>,
    pub path: Option<PathBuf>,
}

impl Volume {
    #[allow(dead_code)]
    pub fn new(
        covers: Vec<Cover>,
        volume: Option<u32>,
        chapters: Vec<Chapter>,
        path: Option<PathBuf>,
    ) -> Self {
        Self {
            covers,
            volume,
            chapters,
            path,
        }
    }

    pub async fn download(&self, multi_bar: &MultiProgress, client: &MangaDexClient) -> Result<()> {
        let style = ProgressStyle::with_template(
            "[{elapsed_precise}] {bar:40.cyan/blue} {pos:>7}/{len:7} {msg}",
        )?;
        let cover_bar = multi_bar.add(
            ProgressBar::new(self.covers.len().try_into()?)
                .with_message("Downloading Covers")
                .with_style(style.clone()),
        );

        if !self.covers.is_empty() {
            cover_bar.inc(0);
            for (index, cover) in self.covers.iter().enumerate() {
                cover.download(index, client).await?;
                cover_bar.inc(1);
            }
            cover_bar.finish_with_message("Downloaded Covers");
        } else {
            cover_bar.finish_and_clear();
        }

        let chapter_bar = multi_bar.add(
            ProgressBar::new(self.chapters.len().try_into()?)
                .with_message("Downloading Chapters")
                .with_style(style.clone()),
        );

        chapter_bar.inc(0);
        for chapter in self.chapters.iter() {
            chapter
                .download_stream(client, multi_bar)
                .await
                .with_context(|| {
                    format!(
                        "Attempting to download chapter {} {:?}, with an ID of {}",
                        chapter.chapter, chapter.sub_chapter, chapter.id
                    )
                })?;
            chapter_bar.inc(1);
        }

        cover_bar.finish_and_clear();
        chapter_bar.finish_and_clear();
        Ok(())
    }
}
