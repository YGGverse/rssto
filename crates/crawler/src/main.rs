mod argument;
mod config;

use anyhow::Result;
use log::{debug, info, warn};
use mysql::Mysql;

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
        for feed in &config.channel {
            debug!("Update `{}`...", feed.url);
            if let Err(e) = crawl(&mut database, feed) {
                warn!("Feed `{}` update failed: `{e}`", feed.url)
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
    use reqwest::blocking::get;
    use rss::Channel;

    let channel = Channel::read_from(&get(channel_config.url.as_str())?.bytes()?[..])?;
    let channel_items = channel.items();
    let channel_items_limit = channel_config.items_limit.unwrap_or(channel_items.len());

    let feed_url = channel_config.url.to_string();
    let channel_id = match db.channels_by_url(&feed_url, Some(1))?.first() {
        Some(result) => result.channel_id,
        None => db.insert_channel(&feed_url)?,
    };

    for channel_item in channel_items.iter().take(channel_items_limit) {
        let guid = match channel_item.guid {
            Some(ref guid) => guid.value.clone(),
            None => {
                warn!("Undefined `guid` field in `{feed_url}`");
                continue;
            }
        };
        let link = match channel_item.guid {
            Some(ref link) => link.value.clone(),
            None => {
                warn!("Undefined `link` field in `{feed_url}`");
                continue;
            }
        };
        let pub_date = match channel_item.pub_date {
            Some(ref pub_date) => match chrono::DateTime::parse_from_rfc2822(pub_date) {
                Ok(t) => t.timestamp(),
                Err(e) => {
                    warn!("Invalid `pub_date` field in `{feed_url}`: `{e}`");
                    continue;
                }
            },
            None => {
                warn!("Undefined `pub_date` field in `{feed_url}`");
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
        }; // @TODO
    }
    Ok(())
}
