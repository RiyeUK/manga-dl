use anyhow::{ensure, Context, Result};
use futures::{stream, StreamExt};
use indicatif::{MultiProgress, ProgressBar, ProgressStyle};
use mangadex_api::{
    utils::download::{chapter::DownloadMode, DownloadElement},
    MangaDexClient,
};
use mangadex_api_schema_rust::{v5::ChapterAttributes, ApiObject};
use std::{
    fs::{create_dir_all, File},
    io::Write,
    path::PathBuf,
};
use uuid::Uuid;

#[derive(Debug, Clone)]
pub struct Chapter {
    pub chapter: u32,
    pub sub_chapter: Option<u32>,
    pub id: Uuid,
    pub pages: u32,
    pub path: Option<PathBuf>,
    pub title: Option<String>,
    pub volume: Option<u32>,
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
    #[allow(dead_code)]
    pub fn new(
        chapter: u32,
        id: Uuid,
        pages: u32,
        path: Option<PathBuf>,
        sub_chapter: Option<u32>,
        title: Option<String>,
        volume: Option<u32>,
    ) -> Self {
        Self {
            chapter,
            id,
            pages,
            path,
            sub_chapter,
            title,
            volume,
        }
    }

    #[allow(dead_code)]
    pub async fn download(&self, client: &MangaDexClient) -> Result<()> {
        // let style = ProgressStyle::with_template(
        //     "[{elapsed_precise}] {bar:40.cyan/blue} {pos:>7}/{len:7} {msg}",
        // )?;
        // let pages_bar = multi_bar.add(
        //     ProgressBar::new(self.pages.try_into()?)
        //         .with_message("Downloading Pages")
        //         .with_style(style.clone()),
        // );
        ensure!(self.path.is_some());
        println!("Creating {}", self.path.as_ref().unwrap().display());
        create_dir_all(self.path.clone().unwrap())?;
        let filenames = client
            .download()
            .chapter(self.id)
            .mode(DownloadMode::Normal)
            .report(true)
            .build()?
            .build_at_home_urls()
            .await?;

        for (index, page) in filenames.iter().enumerate() {
            let (filename, data) = page.download().await.with_context(|| {
                format!(
                    "Attempting to download page {} for chapter {}.{:?}",
                    index, self.chapter, self.sub_chapter
                )
            })?;
            let filename = PathBuf::from(filename);
            let mut result = filename.to_owned();
            result.set_file_name(format!(
                "{:0len$}",
                index,
                len = (self.pages as f64).log10().floor() as usize + 1
            ));
            if let Some(ext) = filename.extension() {
                result.set_extension(ext);
            }
            let mut file = File::create(self.path.clone().unwrap().join(result))?;
            file.write_all(&data.unwrap())?;
        }

        Ok(())
    }

    async fn save_page(
        data: DownloadElement,
        index: usize,
        length: f64,
        path: PathBuf,
    ) -> anyhow::Result<()> {
        create_dir_all(path.clone())?;
        let (filename, bytes) = data;
        let filename = PathBuf::from(filename);
        let mut result = filename.to_owned();
        result.set_file_name(format!(
            "{:0len$}",
            index,
            len = length.log10().floor() as usize + 1
        ));
        if let Some(ext) = filename.extension() {
            result.set_extension(ext);
        }
        let mut file = File::create(path.clone().join(result))?;
        file.write_all(&bytes.unwrap())?;
        Ok(())
    }

    pub async fn download_stream(
        &self,
        client: &MangaDexClient,
        multi_bar: &MultiProgress,
    ) -> Result<Vec<anyhow::Result<()>>> {
        let style = ProgressStyle::with_template(
            "[{elapsed_precise}] {bar:40.cyan/blue} {pos:>7}/{len:7} {msg}",
        )?;
        let page_bar = multi_bar.add(
            ProgressBar::new(1)
                .with_message("Downloading Pages")
                .with_style(style.clone()),
        );
        let file_names = client
            .download()
            .chapter(self.id)
            .mode(DownloadMode::Normal)
            .report(true)
            .build()?
            .build_at_home_urls()
            .await?;

        let len = file_names.len();
        page_bar.set_length(len.try_into()?);
        let mut stream = stream::iter(file_names)
            .enumerate()
            .map(|(index, filename)| async move {
                let data = filename.download().await?;
                Chapter::save_page(data, index + 1, len as f64, self.path.clone().unwrap()).await?;
                Ok(())
            })
            .buffer_unordered(5);
        let mut results = Vec::new();

        while let Some(page) = stream.next().await {
            results.push(page);
            page_bar.inc(1);
        }
        page_bar.finish_and_clear();

        Ok(results)
    }
}
