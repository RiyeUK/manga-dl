// #![allow(unused_imports)]
// #![allow(dead_code)]
// #![allow(unused_variables)]

use anyhow::{bail, Context, Result};

#[allow(unused_imports)]
use clap::Parser;

use mangadex_api_types_rust::Language;
#[allow(unused_imports)]
use std::ops::{RangeFrom, RangeInclusive};
use std::{
    ops::{Range, RangeBounds},
    path::PathBuf,
    str::FromStr,
};

mod int_range;
mod manga;
use manga::get::GetMangaBuilder;

// Example of use
// manga-dl "Loving Yamada at LV999!"
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Cli {
    manga_title: String,

    #[arg(short, long, value_name = "FOLDER")]
    output: PathBuf,

    #[arg(long, value_name = "ID")]
    anilist_id: Option<u32>,

    #[arg(short, long, value_name = "RANGE")]
    chapters: Option<String>,

    // #[arg(short, long)]
    // volumes: Option<Range<u32>>,
    #[arg(long, value_enum)]
    cover_language: Option<Language>,

    #[arg(short, long, value_enum)]
    translated_language: Option<Language>,
}

#[tokio::main]
async fn main() -> Result<()> {
    // Convert all of these into invlusive
    // let range_string_a = "0.."
    // let range_string_b = "0..1"
    // let range_string_c = "..5" //
    // let range_string_d = "0..=5"
    // let range_string_e = "..=5"
    // let range : Range =
    // let cli = Cli::parse();

    // dbg!(cli);
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
