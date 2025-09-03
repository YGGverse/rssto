mod argument;
mod config;

use anyhow::Result;
use argument::Argument;
use chrono::{DateTime, Local};
use clap::Parser;
use config::{Config, Feed};
use log::{debug, info};
use std::{
    env::var,
    fs::{File, create_dir_all, read_to_string},
    io::Write,
    path::PathBuf,
};

fn main() -> Result<()> {
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
    let config: Config = toml::from_str(&read_to_string(argument.config)?)?;

    info!("Crawler started");

    loop {
        debug!("Begin new crawl queue...");

        for feed in &config.feed {
            debug!("Update `{}`...", feed.url);
            crawl(feed)?
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

fn crawl(feed: &Feed) -> Result<()> {
    use reqwest::blocking::get;
    use rss::Channel;

    let channel = Channel::read_from(&get(feed.url.as_str())?.bytes()?[..])?;
    let channel_items = channel.items();
    let channel_items_limit = feed.list_items_limit.unwrap_or(channel_items.len());

    for template in &feed.templates {
        let root = PathBuf::from(template);
        let extension = root.file_name().unwrap().to_string_lossy();

        let index = {
            let mut p = PathBuf::from(&root);
            p.push(format!("index.{extension}"));
            read_to_string(p)?
        };

        let index_item = {
            let mut p = PathBuf::from(&root);
            p.push("index");
            p.push(format!("item.{extension}"));
            read_to_string(p)?
        };

        create_dir_all(&feed.storage)?;
        File::create({
            let mut p = PathBuf::from(&feed.storage);
            p.push(format!("index.{extension}"));
            p
        })?
        .write_all(
            index
                .replace("{title}", channel.title())
                .replace("{description}", channel.description())
                .replace("{link}", channel.link())
                .replace("{language}", channel.language().unwrap_or_default())
                .replace(
                    "{pub_date}",
                    &time(channel.pub_date(), &feed.pub_date_format),
                )
                .replace(
                    "{last_build_date}",
                    &time(channel.last_build_date(), &feed.last_build_date_format),
                )
                .replace("{time_generated}", &time(None, &feed.time_generated_format))
                .replace(
                    "{items}",
                    &channel_items
                        .iter()
                        .take(channel_items_limit)
                        .map(|i| {
                            index_item
                                .replace("{title}", i.title().unwrap_or_default())
                                .replace("{description}", i.description().unwrap_or_default())
                                .replace("{link}", i.link().unwrap_or_default())
                                .replace("{pub_date}", &time(i.pub_date(), &feed.pub_date_format))
                        })
                        .collect::<String>(),
                )
                .as_bytes(),
        )?
    }

    Ok(())
}

fn time(value: Option<&str>, format: &str) -> String {
    match value {
        Some(v) => DateTime::parse_from_rfc2822(v).unwrap(),
        None => Local::now().into(),
    }
    .format(format)
    .to_string()
}
