mod argument;
mod config;

use anyhow::Result;
use log::{debug, info, warn};
use mysql::Mysql;
use reqwest::blocking::get;

fn main() -> Result<()> {
    use argument::Argument;
    use chrono::Local;
    use clap::Parser;
    use std::{env::var, fs::read_to_string};

    if var("RUST_LOG").is_ok() {
        use tracing_subscriber::{EnvFilter, fmt::*};
        struct T;
        impl time::FormatTime for T {
            fn format_time(&self, w: &mut format::Writer<'_>) -> std::fmt::Result {
                write!(w, "{}", Local::now())
            }
        }
        fmt()
            .with_timer(T)
            .with_env_filter(EnvFilter::from_default_env())
            .init()
    }

    let argument = Argument::parse();
    let config: config::Config = toml::from_str(&read_to_string(argument.config)?)?;
    let mut database = Mysql::connect(
        &config.mysql.host,
        config.mysql.port,
        &config.mysql.user,
        &config.mysql.password,
        &config.mysql.database,
    )?;

    info!("Crawler started");
    loop {
        debug!("Begin new crawl queue...");
        for c in &config.channel {
            debug!("Update `{}`...", c.url);
            if let Err(e) = crawl(&mut database, c) {
                warn!("Channel `{}` update failed: `{e}`", c.url)
            }
        }
        debug!("Crawl queue completed");
        if let Some(update) = config.update {
            debug!("Wait {update} seconds to continue...",);
            std::thread::sleep(std::time::Duration::from_secs(update))
        } else {
            return Ok(());
        }
    }
}

fn crawl(db: &mut Mysql, channel_config: &config::Channel) -> Result<()> {
    use rss::Channel;
    use scraper::Selector;

    // shared local helpers
    fn scrape(url: &str, selector: &Selector) -> Result<Option<String>> {
        let document = scraper::Html::parse_document(&get(url)?.text()?);
        Ok(if let Some(first) = document.select(selector).next() {
            Some(first.inner_html())
        } else {
            warn!("Could not scrape requested inner");
            None
        })
    }

    // allocate once
    let channel_url = channel_config.url.to_string();

    let channel_items = match Channel::read_from(&get(channel_config.url.as_str())?.bytes()?[..]) {
        Ok(response) => response.into_items(),
        Err(e) => {
            warn!("Could not parse response from `{channel_url}`: `{e}`");
            return Ok(());
        }
    };

    let channel_items_limit = channel_config.items_limit.unwrap_or(channel_items.len());

    let channel_id = match db.channels_by_url(&channel_url, Some(1))?.first() {
        Some(result) => result.channel_id,
        None => db.insert_channel(&channel_url)?,
    };

    for channel_item in channel_items.iter().take(channel_items_limit) {
        let guid = match channel_item.guid {
            Some(ref guid) => guid.value.clone(),
            None => {
                warn!("Undefined `guid` field in `{channel_url}`");
                continue;
            }
        };
        let link = match channel_item.guid {
            Some(ref link) => link.value.clone(),
            None => {
                warn!("Undefined `link` field in `{channel_url}`");
                continue;
            }
        };
        let pub_date = match channel_item.pub_date {
            Some(ref pub_date) => match chrono::DateTime::parse_from_rfc2822(pub_date) {
                Ok(t) => t.timestamp(),
                Err(e) => {
                    warn!("Invalid `pub_date` field in `{channel_url}`: `{e}`");
                    continue;
                }
            },
            None => {
                warn!("Undefined `pub_date` field in `{channel_url}`");
                continue;
            }
        };
        let channel_item_id = match db
            .channel_items_by_channel_id_guid(channel_id, &guid, Some(1))?
            .first()
        {
            Some(result) => result.channel_item_id,
            None => db.insert_channel_item(
                channel_id,
                pub_date,
                &guid,
                &link,
                if channel_config.persist_item_title {
                    channel_item.title()
                } else {
                    None
                },
                if channel_config.persist_item_description {
                    channel_item.description()
                } else {
                    None
                },
            )?,
        };

        // @TODO preload remote content

        let title = match channel_config.content_title_selector {
            Some(ref selector) => match scrape(&link, selector) {
                Ok(value) => value,
                Err(e) => {
                    warn!("Could not update `title` selector in `{channel_url}`: `{e}`");
                    continue;
                }
            },
            None => None,
        };

        let description = match channel_config.content_description_selector {
            Some(ref selector) => match scrape(&link, selector) {
                Ok(value) => value,
                Err(e) => {
                    warn!("Could not update `description` selector in `{channel_url}`: `{e}`");
                    continue;
                }
            },
            None => None,
        };

        if title.is_none() && description.is_none() {
            continue;
        }

        // @TODO insert content record

        println!("{:?}", description)
    }
    Ok(())
}
