use serde::Deserialize;
use std::path::PathBuf;
use url::Url;

#[derive(Debug, Deserialize)]
pub struct Feed {
    /// RSS feed source
    pub url: Url,

    /// Destination directory
    pub storage: PathBuf,

    /// Path to templates (export formats)
    pub templates: Vec<PathBuf>,

    /// Limit channel items (unlimited by default)
    pub list_items_limit: Option<usize>,

    pub pub_date_format: String,
    pub last_build_date_format: String,
    pub time_generated_format: String,
}

#[derive(Debug, Deserialize)]
pub struct Config {
    pub feed: Vec<Feed>,

    /// Update timeout in seconds
    ///
    /// * None to generate once
    pub update: Option<u64>,
}
