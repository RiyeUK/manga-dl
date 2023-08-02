// #![allow(unused_imports)]
// #![allow(dead_code)]
// #![allow(unused_variables)]

use anyhow::Result;
use mangadex_api_types_rust::Language;
#[allow(unused_imports)]
use std::ops::{RangeFrom, RangeInclusive};

mod manga;
use manga::get::GetMangaBuilder;

#[tokio::main]
async fn main() -> Result<()> {
    let manga = GetMangaBuilder::<RangeFrom<u32>>::default()
        .title("Loving Yamada at LV999! ")
        .anilist_id(109501 as u32)
        // .volumes(0..)
        .chapters(48..)
        .cover_langauge(Language::Japanese)
        .translated_language(Language::English)
        .output("F:/Manga/{title}/")
        .build()?
        .get()
        .await?;

    manga.download().await?;

    Ok(())
}
