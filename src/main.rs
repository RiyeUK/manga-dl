// #![allow(unused_imports)]
// #![allow(dead_code)]
// #![allow(unused_variables)]

use anyhow::Result;
use clap::Parser;

mod int_range;
mod manga;
use manga::get::GetManga;

#[tokio::main]
async fn main() -> Result<()> {
    let manga = GetManga::parse();

    manga.get().await?.download().await?;
    // let manga = GetMangaBuilder::<RangeFrom<u32>>::default()
    //     .title("Loving Yamada at LV999! ")
    //     .anilist_id(109501 as u32)
    //     // .volumes(0..)
    //     .chapters(48..)
    //     .cover_langauge(Language::Japanese)
    //     .translated_language(Language::English)
    //     .output("F:/Manga/{title}/")
    //     .build()?
    //     .get()
    //     .await?;
    // manga.download().await?;

    Ok(())
}
