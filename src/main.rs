use anyhow::{Context, Result};
use indicatif::{MultiProgress, ProgressBar, ProgressStyle};
use mangadex_api_types_rust::Language;
use std::{ops::RangeBounds, str::FromStr, sync::Arc};
use tokio::join;
use uuid::Uuid;

mod manga;
use manga::Manga;

struct MangaBuilder<R: RangeBounds<u32>> {
    // anilist_id: Option<String>,
    id: Option<Uuid>,
    volumes: Option<R>,
    chapters: Option<R>,
    download_covers: bool,
    cover_langauge: Option<Language>,
    translated_language: Option<Language>,
}

impl<R: RangeBounds<u32>> MangaBuilder<R> {
    fn new() -> Self {
        Self {
            id: None,
            volumes: None,
            chapters: None,
            download_covers: false,
            cover_langauge: None,
            translated_language: None,
        }
    }

    fn volumes(&mut self, volumes: R, language: Language) -> &mut Self {
        self.volumes = Some(volumes);
        self.translated_language = Some(language);
        self
    }

    fn covers(&mut self, language: Language) -> &mut Self {
        self.cover_langauge = Some(language);
        self.download_covers = true;
        self
    }

    fn manga_id(&mut self, id: Uuid) -> &mut Self {
        self.id = Some(id);
        self
    }

    async fn build(&mut self) -> Result<Manga> {
        // Create a multi-progress bar
        let multi_pb = MultiProgress::new();

        // Create Progress bars, we will update with a new length if required
        let metadata_pb = multi_pb.add(ProgressBar::new(1));
        let chapters_pb = multi_pb.add(ProgressBar::new(1));
        let covers_pb = multi_pb.add(ProgressBar::new(1));

        let style = ProgressStyle::with_template("{bar:40.cyan/blue} {pos:>7}/{len:7} {msg}")?;

        metadata_pb.set_style(style.clone());
        chapters_pb.set_style(style.clone());
        covers_pb.set_style(style.clone());

        let manga_arc = Arc::new(Manga::new(self.id.context("Missing Manga ID")?));

        let metadata_task = tokio::spawn({
            let manga = manga_arc.clone();
            async move {
                manga.get_metadata(&metadata_pb).await?;
                anyhow::Ok(())
            }
        });

        // We need to split these out so they can go to different threads
        let translated_language = self.translated_language.unwrap_or(Language::English);
        let cover_language = self.cover_langauge.unwrap_or(Language::Japanese);

        let chapters_task = tokio::spawn({
            let manga_arc = manga_arc.clone();
            async move {
                manga_arc
                    .get_chapters(translated_language, &chapters_pb)
                    .await?;
                anyhow::Ok(())
            }
        });

        let covers_task = tokio::spawn({
            let manga = manga_arc.clone();
            async move {
                manga
                    .get_covers(cover_language, &covers_pb)
                    .await?;
                anyhow::Ok(())
            }
        });

        // Await the completion of all tasks
        let (metadata_result, chapters_result, covers_result) =
            join!(metadata_task, chapters_task, covers_task);

        // Handle errors from each task (optional, based on your requirements)
        metadata_result??;
        chapters_result??;
        covers_result??;

        let mut manga = Arc::try_unwrap(manga_arc).expect("Unable to unwrap ARC");

        // Wait for all progress bars to finish and print the summary
        if let Some(volumes) = &self.volumes {
            manga.volumes(volumes);
        }

        if let Some(chapters) = &self.chapters {
            manga.chapters(chapters);
        }

        Ok(manga)
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    MangaBuilder::new()
        // .anilist_id()
        // .search("Manga Title")
        .manga_id(Uuid::from_str("643561e6-5c27-4382-95d3-8e84894a3fb6")?)
        .volumes(0.., Language::English)
        .covers(Language::Japanese)
        .build()
        .await?
        .download()
        .await?;

    Ok(())
}

// async fn download_covers(manga_id: Uuid, manga_name: &str) -> Result<()> {
//     let client = MangaDexClient::default();
//     let data = client
//         .cover()
//         .list()
//         .add_manga_id(&manga_id)
//         .limit(100 as u32)
//         .locale(Language::Japanese)
//         .include(ReferenceExpansionResource::CoverArt)
//         // .include(ReferenceExpansionResource::Author)
//         .build()?
//         .send()
//         .await?;
//     let mut seen_volumes: HashMap<u32, u32> = HashMap::new();
//     for item in data.data.into_iter() {
//         if let Some(volume) = item.attributes.volume {
//             match volume.parse::<f32>() {
//                 core::result::Result::Ok(vol) => {
//                     let output_dir = format!("{}/{} Vol. {}", manga_id, manga_name, (vol as u32));
//                     create_dir_all(&output_dir)?;
//                     let count = seen_volumes.entry(vol as u32).or_insert(0);
//                     let cover_name = if vol.fract() != 0.0 {
//                         format!("cover{}_{count}", (vol.fract() * 10.0) as u32)
//                     } else {
//                         format!("cover{count}")
//                     };
//                     *count += 1;
//                     if let (filename, Some(bytes)) = client
//                         .download()
//                         .cover()
//                         .build()?
//                         .via_cover_id(item.id)
//                         .await?
//                     {
//                         let extension = get_file_extension(filename.as_str());
//                         let mut file =
//                             File::create(format!("{}/{}{}", output_dir, cover_name, extension))?;
//                         file.write_all(&bytes)?;
//                         println!("Downloaded {filename}");
//                     }
//                 }
//                 Err(e) => {
//                     println!("Invalid Cover Volume number: {volume} {}", e);
//                 }
//             }
//         }
//     }
//     // dbg!(data);
//     Ok(())
// }
// async fn download_manga(manga_id: Uuid, chapter_offset: f32, manga_name: &str) -> Result<()> {
//     let client = MangaDexClient::default();
//     let offset: u32 = 63;
//     let manga = client
//         .manga()
//         .feed()
//         .manga_id(&manga_id)
//         .add_translated_language(Language::English)
//         .offset(offset)
//         .limit(500 as u32)
//         .order(MangaFeedSortOrder::Chapter(OrderDirection::Ascending))
//         .build()?
//         .send()
//         .await?;

//     let mut grabbed_chapters: Vec<String> = Vec::new();
//     for (index, chapter) in manga?.data.into_iter().enumerate() {
//         print!("({}) ", index + (offset as usize));
//         let vol = chapter.attributes.volume.expect("No Volume!?");
//         let ch = chapter.attributes.chapter.expect("No Chapter!?");

//         match ch.parse::<f32>() {
//             core::result::Result::Ok(n) => {
//                 if n < chapter_offset {
//                     println!("Skipping Chapter");
//                     continue;
//                 }
//             }
//             Err(e) => println!("Chapter is NAN? {e}"),
//         }
//         let mut title = chapter.attributes.title;
//         title.retain(|c| !r#"\"<>:\\/|?*"#.contains(c));
//         if title.ends_with("...") {
//             title.truncate(title.len() - 3);
//         }
//         if grabbed_chapters.contains(&ch) {
//             println!("Already Grabbed, Skipping");
//         } else {
//             grabbed_chapters.push(ch.clone());
//             println!("Downloading Vol. {vol} Ch. {ch} {title}");
//             let chapter_directory = if title.is_empty() {
//                 format!("{}/{} Vol. {}/Ch.{}", manga_id, manga_name, vol, ch.trim())
//             } else {
//                 format!(
//                     "{}/{} Vol. {}/Ch.{} - {}",
//                     manga_id,
//                     manga_name,
//                     vol,
//                     ch.trim(),
//                     title.trim()
//                 )
//             };

//             download_chapter(chapter.id, client.clone(), &chapter_directory).await?;
//         }
//     }

//     Ok(())
// }

// async fn download_chapter(
//     chapter_id: Uuid,
//     client: MangaDexClient,
//     output_dir: &str,
// ) -> Result<()> {
//     create_dir_all(output_dir)?;
//     let download = client
//         .download()
//         .chapter(chapter_id)
//         .mode(DownloadMode::Normal)
//         .report(true)
//         .build()?;
//     let chapter_files = download.download_stream().await?;
//     pin!(chapter_files);
//     while let Some((data, index, len, _)) = chapter_files.next().await {
//         print!("{index} - {len} : ");
//         if let core::result::Result::Ok(resp) = data {
//             let (filename, bytes_) = resp;
//             if let Some(bytes) = bytes_ {
//                 let extension = get_file_extension(filename.as_str());
//                 let filename = format!(
//                     "{:0len$}",
//                     index,
//                     len = (len as f64).log10().floor() as usize + 1
//                 );
//                 // println!("Creating file {}/{}{}", output_dir, filename, extension);
//                 let mut file = File::create(format!("{}/{}{}", output_dir, filename, extension))?;
//                 file.write_all(&bytes)?;
//                 println!("Downloaded {filename}");
//             } else {
//                 println!("Skipped {filename}");
//             }
//         } else if let core::result::Result::Err(resp) = data {
//             println!("{:#?}", resp);
//         }
//     }

//     Ok(())
// }

// fn get_file_extension(filename: &str) -> &str {
//     if let Some(dot_index) = filename.rfind('.') {
//         &filename[dot_index..]
//     } else {
//         ""
//     }
// }
