use anyhow::Result;
use indicatif::{ProgressBar, ProgressStyle};
use mangadex_api::MangaDexClient;
use mangadex_api_schema_rust::{v5::MangaAttributes, ApiObject};
use mangadex_api_types_rust::{Language, MangaFeedSortOrder, OrderDirection};
use std::{ops::RangeBounds, path::PathBuf, str::FromStr};
use uuid::Uuid;

mod chapter;
mod cover;

use chapter::Chapter;
use cover::Cover;

#[derive(Debug)]
pub struct Manga {
    id: Uuid,
    client: MangaDexClient,
    title: Option<String>,
    author: Option<String>,
    chapters: Vec<Chapter>,
    covers: Vec<Cover>,
    path: Option<PathBuf>,
}

impl<T> TryFrom<ApiObject<MangaAttributes, T>> for Manga {
    type Error = anyhow::Error;

    fn try_from(value: ApiObject<MangaAttributes, T>) -> Result<Self, Self::Error> {
        todo!()
    }
}

impl Manga {
    pub fn new(id: Uuid) -> Self {
        Self {
            id,
            client: MangaDexClient::default(),
            title: None,
            author: None,
            chapters: Vec::new(),
            covers: Vec::new(),
            path: None,
        }
    }

    pub fn from_str(uuid_str: &str) -> anyhow::Result<Manga> {
        Ok(Manga::new(Uuid::from_str(&uuid_str)?))
    }

    pub async fn get_covers(
        &self,
        cover_language: Language,
        pb: &ProgressBar,
    ) -> Result<Vec<Cover>> {
        // let cover_data = self
        //     .client
        //     .cover()
        //     .add_manga_id()
        todo!()
    }

    pub async fn get_metadata(&self, pb: &ProgressBar) -> anyhow::Result<()> {
        todo!();
    }

    pub async fn get_chapters(
        &self,
        translated_language: Language,
        pb: &ProgressBar,
    ) -> Result<Vec<Chapter>> {
        const MAX_LIMIT: u32 = 500; // This is the max that mangadex allows
        let mut offset = 0;
        let mut chapters = Vec::<Chapter>::new();
        loop {
            let chapter_data = self
                .client
                .manga()
                .feed()
                .manga_id(&self.id)
                .add_translated_language(translated_language)
                .offset(offset)
                .limit(MAX_LIMIT)
                .order(MangaFeedSortOrder::Chapter(OrderDirection::Ascending))
                .build()?
                .send()
                .await??;

            // Update the length of the bar with the number of times we need to paginate
            pb.set_length((chapter_data.total / chapter_data.limit) as u64 + 1);

            for chapter in chapter_data.data {
                chapters.push(chapter.try_into()?);
            }

            if chapter_data.limit + chapter_data.offset > chapter_data.total {
                break;
            }

            pb.inc(1);

            // Update the offset and paginate
            offset += MAX_LIMIT;
        }

        // Chapters should be sorted because we called the api with an order
        // we can use this fact to remove duplicate chapters (from different scan groups)
        // because they *should* be consecutive.
        // This will need testing to confirm this is working as expected
        pb.finish_with_message(format!("Grabbed {} chapters.", chapters.len()));
        Ok(chapters)
    }

    pub fn volumes(&mut self, volumes: &impl RangeBounds<u32>) -> &mut Self {
        self.chapters.retain(|chapter| {
            if let Some(volume) = chapter.volume {
                volumes.contains(&volume)
            } else {
                false
            }
        });
        self
    }
    pub fn chapters(&mut self, chapters: &impl RangeBounds<u32>) {
        self.chapters
            .retain(|chapter| chapters.contains(&chapter.chapter));
    }

    pub async fn download(&mut self) -> Result<()> {
        todo!()
    }
}
