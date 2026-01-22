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
}

#[derive(Debug, PartialEq, Eq, FromRow)]
pub struct ChannelItemDescription {
    pub channel_item_description_id: u64,
    pub channel_item_id: u64,
    pub provider_id: Option<u64>,
    pub title: Option<String>,
    pub description: Option<String>,
}

#[derive(Debug, PartialEq, Eq, FromRow)]
pub struct ChannelItemContent {
    pub channel_item_content_id: u64,
    pub channel_item_id: u64,
}

#[derive(Debug, PartialEq, Eq, FromRow)]
pub struct ChannelItemContentDescription {
    pub channel_item_content_description_id: u64,
    pub channel_item_content_id: u64,
    pub provider_id: Option<u64>,
    pub title: Option<String>,
    pub description: Option<String>,
}

#[derive(Debug, PartialEq, Eq, FromRow)]
pub struct Provider {
    pub provider_id: u64,
    pub name: String,
}

#[derive(Debug, PartialEq, Eq, FromRow)]
pub struct Image {
    pub image_id: u64,
    pub provider_id: Option<u64>,
    /// Keep image unique by comparing its data hash
    pub sha256: String,
    /// Original `src` tag value to post-replacing
    pub src: Option<String>,
    /// Resolved absolute URL
    pub url: Option<String>,
    /// Image data, MEDIUMBLOB (16M)
    pub data: Vec<u8>,
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
