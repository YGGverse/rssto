use scraper::Selector;
use serde::Deserialize;
use url::Url;

#[derive(Debug, Deserialize)]
pub struct Mysql {
    pub database: String,
    pub host: String,
    pub password: String,
    pub port: u16,
    pub username: String,
}

#[derive(Debug, Deserialize)]
pub struct Channel {
    /// RSS feed source
    pub url: Url,
    /// Limit latest channel items to crawl (unlimited by default)
    pub items_limit: Option<usize>,
    /// Save Channel item title in the database (currently not in use)
    pub persist_item_title: bool,
    /// Save Channel item description in the database (currently not in use)
    pub persist_item_description: bool,
    /// Scrape title by CSS selector
    /// * None to use Channel item title if exists or fail to continue
    pub content_title_selector: Option<Selector>,
    /// Scrape description by CSS selector
    /// * None to use Channel item description if exists or fail to continue
    pub content_description_selector: Option<Selector>,
    /// Allowed tags
    /// * empty to strip all tags (default)
    pub allowed_tags: std::collections::HashSet<String>,
    /// Preload content images locally if `Some`
    /// * currently stored in the database
    pub persist_images_selector: Option<Selector>,
}

#[derive(Debug, Deserialize)]
pub struct Config {
    pub mysql: Mysql,
    pub channel: Vec<Channel>,
    /// Channels update timeout in seconds
    /// * None to generate once
    pub update: Option<u64>,
}
