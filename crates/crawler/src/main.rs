mod argument;
mod config;

use anyhow::{Result, bail};
use log::{debug, info, warn};
use reqwest::blocking::get;
use url::Url;

fn main() -> Result<()> {
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

    let argument = argument::Argument::parse();
    let config: config::Config = toml::from_str(&read_to_string(argument.config)?)?;
    let db = mysql::Database::pool(
        &config.mysql.host,
        config.mysql.port,
        &config.mysql.username,
        &config.mysql.password,
        &config.mysql.database,
    )?;

    info!("Crawler started");
    loop {
        debug!("Begin new crawl queue...");
        for c in &config.channel {
            debug!("Update `{}`...", c.url);
            let mut tx = db.transaction()?;
            match crawl(&mut tx, c) {
                Ok(()) => tx.commit()?,
                Err(e) => {
                    warn!("Channel `{}` update failed: `{e}`", c.url);
                    tx.rollback()?
                }
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

fn crawl(tx: &mut mysql::Transaction, channel_config: &config::Channel) -> Result<()> {
    use std::collections::HashSet;

    /// Removes all tags from `html` excluding `allowed_tags` or all if None
    fn strip_tags(html: &str, allowed_tags: Option<&HashSet<String>>) -> String {
        ammonia::Builder::new()
            .tags(allowed_tags.map_or(HashSet::new(), |a| a.iter().map(|t| t.as_str()).collect()))
            .clean(html)
            .to_string()
    }

    let channel_url = channel_config.url.to_string(); // allocate once

    let channel_id = match tx.channel_id_by_url(&channel_url)? {
        Some(channel_id) => channel_id,
        None => {
            let channel_id = tx.insert_channel(&channel_url)?;
            info!("Register new channel #{channel_id} ({channel_url})");
            channel_id
        }
    };

    let channel_items =
        match rss::Channel::read_from(&get(channel_config.url.as_str())?.bytes()?[..]) {
            Ok(channel) => {
                if channel_config.persist_description {
                    let channel_description_id = tx.insert_channel_description(
                        channel_id,
                        None,
                        Some(strip_tags(channel.title(), None)),
                        Some(strip_tags(
                            channel.description(),
                            Some(&channel_config.allowed_tags),
                        )),
                    )?;
                    debug!("Save channel description #{channel_description_id}")
                }
                channel.into_items()
            }
            Err(e) => bail!("Could not parse response: `{e}`"),
        };

    let channel_items_limit = channel_config.items_limit.unwrap_or(channel_items.len());

    for channel_item in channel_items.iter().take(channel_items_limit) {
        let guid = match channel_item.guid {
            Some(ref guid) => guid.value.as_ref(),
            None => bail!("Undefined `guid` field"),
        };
        let (link, base) = match channel_item.link {
            Some(ref link) => (link, Url::parse(link)?),
            None => bail!("Undefined `link` field"),
        };
        let pub_date = match channel_item.pub_date {
            Some(ref pub_date) => match chrono::DateTime::parse_from_rfc2822(pub_date) {
                Ok(t) => t.timestamp(),
                Err(e) => bail!("Invalid `pub_date` field: `{e}`"),
            },
            None => bail!("Undefined `pub_date`"),
        };
        if tx.channel_items_total_by_channel_id_guid(channel_id, guid)? > 0 {
            debug!("Channel item `{guid}` already exists, skipped.");
            continue; // skip next steps as processed
        }
        let channel_item_id = tx.insert_channel_item(channel_id, pub_date, guid, link)?;
        info!("Register new channel item #{channel_item_id} ({link})");
        if channel_config.persist_item_description {
            let channel_item_description_id = tx.insert_channel_item_description(
                channel_item_id,
                None,
                channel_item.title().map(|s| strip_tags(s, None)),
                channel_item
                    .description()
                    .map(|s| strip_tags(s, Some(&channel_config.allowed_tags))),
            )?;
            debug!("Save channel item description #{channel_item_description_id}")
        }
        // preload remote content..
        if !channel_config.scrape_item_content {
            continue;
        }
        let channel_item_content_id = tx.insert_channel_item_content(channel_item_id)?;
        info!("Add new content record #{channel_item_content_id}");

        let html = scraper::Html::parse_document(&get(link)?.text()?);
        let description = match channel_config.scrape_item_content_description_selector {
            Some(ref selector) => match html.select(selector).next() {
                Some(description) => Some(strip_tags(
                    &description.inner_html(),
                    Some(&channel_config.allowed_tags),
                )),
                None => bail!("Could not scrape `description` selector from `{link}`"),
            },
            None => None,
        };
        let channel_item_content_description_id = tx.insert_channel_item_content_description(
            channel_item_content_id,
            None,
            match channel_config.scrape_item_content_title_selector {
                Some(ref selector) => match html.select(selector).next() {
                    Some(title) => Some(strip_tags(&title.inner_html(), None)),
                    None => bail!("Could not scrape `title` selector from `{link}`"),
                },
                None => None,
            }
            .as_ref()
            .map(|s| s.trim()),
            description.as_ref().map(|s| s.trim()),
        )?;
        debug!("Save channel item content description #{channel_item_content_description_id}");
        // persist images if enabled
        if let Some(ref selector) = channel_config.persist_images_selector {
            use sha2::{Digest, Sha256};
            if description.is_none() {
                bail!("Field `description` is required to scrape images from `{link}`")
            }
            for element in scraper::Html::parse_document(&description.unwrap()).select(selector) {
                if let Some(src) = element.value().attr("src") {
                    let absolute = match Url::parse(src) {
                        Ok(url) => url,
                        Err(e) => {
                            if e == url::ParseError::RelativeUrlWithoutBase {
                                let absolute = base.join(link)?;
                                debug!("Convert relative image link `{link}` to `{absolute}`");
                                absolute
                            } else {
                                bail!("Could not parse URL from img source: `{e}`")
                            }
                        }
                    };
                    let url = absolute.as_str();
                    let data = get(url)?.bytes()?;
                    let hash = format!("{:x}", Sha256::digest(&data));

                    let image_id = match tx.image_id_by_sha256(&hash)? {
                        Some(image_id) => image_id,
                        None => {
                            let image_id = tx.insert_image(&hash, Some(src), Some(url), &data)?;
                            info!("Persist new image #{image_id} (`{absolute}`)");
                            image_id
                        }
                    };
                    let channel_item_content_image_id =
                        tx.insert_channel_item_content_image(channel_item_content_id, image_id)?;
                    debug!("Add content image relationship #{channel_item_content_image_id}");
                    let uri = format!("/image/{image_id}");
                    tx.replace_channel_item_content_description(
                        channel_item_content_description_id,
                        src,
                        &uri,
                    )?;
                    debug!("Replace content image in description from `{src}` to `{uri}`")
                }
            }
        }
    }
    Ok(())
}
