use scraper::Selector;
use serde::Deserialize;
use url::Url;

#[derive(Debug, Deserialize)]
pub struct Mysql {
    pub database: String,
    pub host: String,
    pub password: String,
    pub port: u16,
    pub user: String,
}

#[derive(Debug, Deserialize)]
pub struct Channel {
    /// RSS feed source
    pub url: Url,
    /// Limit channel items (unlimited by default)
    pub items_limit: Option<usize>,
    /// Save item title
    pub persist_item_title: bool,
    /// Save item description
    pub persist_item_description: bool,
    /// Scrape title by CSS selector
    /// * None to ignore
    pub content_title_selector: Option<Selector>,
    /// Scrape description by CSS selector
    /// * None to ignore
    pub content_description_selector: Option<Selector>,
}

#[derive(Debug, Deserialize)]
pub struct Config {
    pub mysql: Mysql,
    pub channel: Vec<Channel>,
    /// Channels update timeout in seconds
    /// * None to generate once
    pub update: Option<u64>,
}
