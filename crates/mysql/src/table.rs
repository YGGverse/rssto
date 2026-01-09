use mysql::prelude::FromRow;

#[derive(Debug, PartialEq, Eq, FromRow)]
pub struct Channel {
    pub channel_id: u64,
    pub url: String,
}

#[derive(Debug, PartialEq, Eq, FromRow)]
pub struct ChannelItem {
    pub channel_item_id: u64,
    pub channel_id: u64,
    pub pub_date: i64,
    pub guid: String,
    pub link: String,
    pub title: Option<String>,
    pub description: Option<String>,
}

#[derive(Debug, PartialEq, Eq, FromRow)]
pub struct Content {
    pub content_id: u64,
    pub channel_item_id: u64,
    /// None if the original `title` and `description` values
    /// parsed from the channel item on crawl
    pub provider_id: Option<u64>,
    pub title: String,
    pub description: String,
}

#[derive(Debug, PartialEq, Eq, FromRow)]
pub struct Provider {
    pub provider_id: u64,
    pub name: String,
}

#[derive(Debug, PartialEq, Eq, FromRow)]
pub struct Image {
    pub image_id: u64,
    pub source: String,
    pub data: Vec<u8>,
}

/// Includes joined `image` table members
#[derive(Debug, PartialEq, Eq, FromRow)]
pub struct ContentImage {
    pub content_image_id: u64,
    pub content_id: u64,
    pub image_id: u64,
    // Image members (JOIN)
    pub data: Vec<u8>,
    pub source: String,
}

pub enum Sort {
    Asc,
    Desc,
}

impl std::fmt::Display for Sort {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Self::Asc => write!(f, "ASC"),
            Self::Desc => write!(f, "DESC"),
        }
    }
}
