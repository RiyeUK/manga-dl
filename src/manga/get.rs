use super::{chapter::Chapter, cover::Cover, mangadata::MangaData, volume::Volume, Manga};
use anyhow::{Context, Result};
use derive_builder::Builder;
use mangadex_api::MangaDexClient;
use mangadex_api_types_rust::{
    Language, MangaFeedSortOrder, OrderDirection, ReferenceExpansionResource,
};
use std::{collections::HashMap, ops::RangeBounds, path::PathBuf};
use uuid::Uuid;

#[derive(Builder, Debug)]
#[builder(public)]
#[builder(setter(into, strip_option))]
pub struct GetManga<R>
where
    R: RangeBounds<u32>,
{
    /// Anilist ID use in conjunction with title
    #[builder(default = "None")]
    pub anilist_id: Option<u32>,

    #[builder(setter(custom), default = "None")]
    pub chapters: Option<R>,

    /// The language for the covers defaults, if not set we do not download covers
    #[builder(default = "None")]
    pub cover_langauge: Option<Language>,

    #[builder(default = "None")]
    pub id: Option<Uuid>,

    /// Required
    // The base file path of where the files should be saved
    pub output: PathBuf,

    #[builder(default = "None")]
    pub title: Option<String>,

    /// Required
    /// The language we get the manga translated into
    pub translated_language: Language,

    #[builder(setter(custom), default = "None")]
    pub volumes: Option<R>,
}

impl<R: Default> Default for GetManga<R>
where
    R: RangeBounds<u32>,
{
    fn default() -> Self {
        Self {
            anilist_id: None,
            chapters: None,
            cover_langauge: None,
            id: None,
            output: PathBuf::default(),
            title: None,
            translated_language: Language::English,
            volumes: None,
        }
    }
}

#[allow(dead_code)]
impl<R: RangeBounds<u32>> GetMangaBuilder<R> {
    /// Sets the range of volumes we wish to download. Is overwritten by chapters.
    pub fn volumes(&mut self, value: R) -> &mut Self {
        self.volumes = Some(Some(value));
        self
    }

    /// Sets the range of chapters we wish to download. Overwrites volumes.
    pub fn chapters(&mut self, value: R) -> &mut Self {
        self.chapters = Some(Some(value));
        self
    }
}

impl<R> GetManga<R>
where
    R: RangeBounds<u32>,
{
    pub async fn get(&self) -> Result<Manga> {
        let client = MangaDexClient::default();

        let id: Uuid = self.id.unwrap_or(self.search(&client).await?);
        let metadata = self.fetch_metadata(&client, &id).await?;
        let path: PathBuf = self
            .output
            .to_str()
            .context("Missing Output!")?
            .replace(
                "{title}",
                metadata
                    .title
                    .get(&Language::English)
                    .context("No English title!")?,
            )
            .into();
        let volumes = self.fetch_chapters(&client, &id, &path).await?;

        Ok(Manga {
            client,
            id,
            metadata,
            volumes,
            path,
        })
    }

    /// Search for the Mangadex UUID by searching and then checking against the
    /// AnilistID if present.
    async fn search(&self, client: &MangaDexClient) -> Result<Uuid> {
        println!("Searching for Manga ID...");

        let search_data = client
            .search()
            .manga()
            .title(
                &*self
                    .title
                    .clone()
                    .context("Attempted to search without a title!")?,
            )
            .available_translated_language(vec![self.translated_language])
            .build()?
            .send()
            .await?;

        if let Some(anilist) = self.anilist_id {
            let id = search_data.data.iter().find_map(|manga| {
                if manga
                    .attributes
                    .links
                    .as_ref()
                    .map(|links| links.anilist.as_ref())
                    == Some(Some(&anilist.to_string()))
                {
                    Some(manga.id)
                } else {
                    None
                }
            });
            println!("Found Manga ID of {}", id.unwrap().to_string());
            id.with_context(|| format!("No Manga Found with anilist id {:?}", anilist))
        } else {
            Ok(search_data.data.first().unwrap().id)
        }
    }

    async fn fetch_covers(
        &self,
        client: &MangaDexClient,
        id: &Uuid,
        path: &PathBuf,
    ) -> Result<HashMap<Option<u32>, Vec<Cover>>> {
        println!("Fetching Covers...");

        let mut covers: Vec<Cover> = Vec::new();
        let mut offset = 0;
        const COVER_LIMIT: u32 = 10;
        loop {
            let cover_data = client
                .cover()
                .list()
                .limit(COVER_LIMIT)
                .offset(offset)
                .manga_ids(vec![id.clone()])
                .locale(
                    self.cover_langauge
                        .context("Called fetch_covers without a cover language set!")?,
                )
                .build()?
                .send()
                .await?;

            // dbg!(cover_data.clone());

            for cover in cover_data.data {
                if let Ok(cover_item) = cover.try_into() {
                    covers.push(cover_item);
                }
            }

            if cover_data.limit + cover_data.offset > cover_data.total {
                // We do not need to paginate so
                break;
            }

            offset += COVER_LIMIT;
        }

        for cover in &mut covers {
            cover.path = Some(path.clone().join(format!(
                "Vol. {}",
                cover
                    .volume
                    .map_or("None".to_string(), |num| format!("{:?}", num))
            )));
        }

        let mut covers_by_volume: HashMap<Option<u32>, Vec<Cover>> = HashMap::new();

        for cover in &covers {
            covers_by_volume
                .entry(cover.volume)
                .or_insert(Vec::new())
                .push(cover.clone());
        }

        println!("Got {} covers!", covers.len());
        Ok(covers_by_volume)
    }

    async fn fetch_metadata(&self, client: &MangaDexClient, id: &Uuid) -> Result<MangaData> {
        println!("Fetching Manga Metadata...");

        let manga_data = client
            .manga()
            .get()
            .includes(vec![ReferenceExpansionResource::Author])
            .manga_id(id)
            .build()?
            .send()
            .await?;

        println!("Metadata loaded.");
        Ok(manga_data.data.into())
    }

    async fn fetch_chapters(
        &self,
        client: &MangaDexClient,
        id: &Uuid,
        path: &PathBuf,
    ) -> Result<Vec<Volume>> {
        const CHAPTER_LIMIT: u32 = 500; // Max that the mangadex api allows
        let mut offset = 0;
        let mut volumes: HashMap<Option<u32>, Vec<Chapter>> = HashMap::new();
        let mut count = 0;
        loop {
            let chapters_data = client
                .manga()
                .feed()
                .manga_id(id)
                .add_translated_language(self.translated_language)
                .offset(offset)
                .limit(CHAPTER_LIMIT)
                .order(MangaFeedSortOrder::Chapter(OrderDirection::Ascending))
                .build()?
                .send()
                .await??;

            for chapter in chapters_data.data {
                let mut chapter: Chapter = chapter.try_into()?;
                if chapter.pages <= 0 {
                    // TODO: Add logic to detect dulicates
                    continue;
                }

                let mut ch_path = path.clone();
                ch_path.push(format!(
                    "Vol. {}",
                    chapter
                        .volume
                        .map_or("None".to_string(), |num| format!("{:?}", num))
                ));

                ch_path.push(PathBuf::from(
                    match (&chapter.title, &chapter.sub_chapter) {
                        (Some(title), Some(sub)) => {
                            format!("Ch. {}.{} - {}", chapter.chapter, sub, title)
                        }
                        (Some(title), None) => format!("Ch. {} - {}", chapter.chapter, title),
                        (None, Some(sub)) => format!("Ch. {}.{}", chapter.chapter, sub),
                        (None, None) => format!("Ch. {}", chapter.chapter),
                    },
                ));

                chapter.path = Some(ch_path);

                // Check to see we should save this chapter
                // or exit early
                if match (&self.chapters, &self.volumes) {
                    // Chapter ranges takes precendence over volumes
                    (Some(ch_range), _) => ch_range.contains(&chapter.chapter),
                    (None, Some(vol_range)) => chapter
                        .volume
                        .map_or(false, |volume| vol_range.contains(&volume)),
                    // If both are not set we download all chapters
                    // This is the same as if chapters = .. or 0..
                    // But not if Volumes was set
                    (None, None) => true,
                } {
                    count += 1;
                    // Adds the current chapter to the hashmap based on the given volume
                    volumes
                        .entry(chapter.volume)
                        .or_insert(vec![])
                        .push(chapter);
                } else {
                    continue;
                }
            }

            if chapters_data.limit + chapters_data.offset > chapters_data.total {
                // We do not need to paginate so
                break;
            }

            // Update the offset and paginate
            offset += CHAPTER_LIMIT;
        }
        println!(
            "Got {} chapters over {} volumes",
            count,
            volumes.keys().len()
        );

        let covers: HashMap<Option<u32>, Vec<Cover>> = if self.cover_langauge.is_some() {
            self.fetch_covers(&client, &id, &path).await?
        } else {
            HashMap::new()
        };

        let mut volumes_list: Vec<Volume> = volumes
            .into_iter()
            .map(|(volume, chapters)| Volume {
                covers: covers.get(&volume).cloned().unwrap_or(Vec::new()),
                chapters,
                volume,
                path: Some(path.clone().join(format!(
                    "Vol. {}",
                    volume.map_or("None".to_string(), |num| format!("{:?}", num))
                ))),
            })
            .collect();
        volumes_list.sort_by_key(|v| v.volume);
        Ok(volumes_list)
    }
}