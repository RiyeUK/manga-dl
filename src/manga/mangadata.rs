use mangadex_api_schema_rust::{
    v5::{LocalizedString, MangaAttributes, RelatedAttributes},
    ApiObject,
};

/// Required Manga metadata
#[derive(Debug, Clone, Default)]
pub struct MangaData {
    pub alt_titles: Vec<LocalizedString>,
    pub authors: Vec<String>,
    pub title: LocalizedString,
}

impl MangaData {
    #[allow(dead_code)]
    pub fn new(
        alt_titles: Vec<LocalizedString>,
        authors: Vec<String>,
        title: LocalizedString,
    ) -> Self {
        Self {
            alt_titles,
            authors,
            title,
        }
    }
}

impl From<ApiObject<MangaAttributes>> for MangaData {
    fn from(value: ApiObject<MangaAttributes>) -> Self {
        let authors: Vec<String> = value
            .relationships
            .into_iter()
            .filter_map(|rel| match rel.attributes {
                Some(RelatedAttributes::Author(data)) => Some(data.name),
                _ => None,
            })
            .collect();

        MangaData {
            title: value.attributes.title,
            alt_titles: value.attributes.alt_titles,
            authors,
        }
    }
}
