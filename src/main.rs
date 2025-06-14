mod argument;
mod debug;
mod format;
mod target;
mod time;

use anyhow::{Result, bail};
use argument::Argument;
use debug::Debug;
use format::Format;
use format::Type;
use target::Target;
use time::Time;

fn main() -> Result<()> {
    use clap::Parser;
    use std::{thread::sleep, time::Duration};

    let argument = Argument::parse();

    // parse argument values once
    let debug = Debug::init(&argument.debug)?;
    let format = Format::init(&argument.format, &argument.template)?;
    let target = Target::init(&argument.target)?;
    let time = Time::init(argument.time_format);

    // validate some targets
    if argument.source.len() != argument.target.len() {
        bail!("Targets quantity does not match sources!")
    }

    debug.info("Crawler started");

    loop {
        debug.info("Begin new crawl queue...");
        for (i, s) in argument.source.iter().enumerate() {
            debug.info(&format!("Update {s}..."));
            crawl((s, i), &format, &target, &time, &argument.limit)?;
        }
        debug.info(&format!(
            "Crawl queue completed, wait {} seconds to continue...",
            argument.update
        ));
        sleep(Duration::from_secs(argument.update));
    }
}

fn crawl(
    source: (&str, usize),
    format: &Format,
    target: &Target,
    time: &Time,
    limit: &Option<usize>,
) -> Result<()> {
    use reqwest::blocking::get;
    use rss::Channel;
    use std::{fs::File, io::Write};

    let c = Channel::read_from(&get(source.0)?.bytes()?[..])?;
    let i = c.items();
    let l = limit.unwrap_or(i.len());

    for f in format.get() {
        match f {
            Type::Html(template) => File::create(target.index(source.1, "html"))?.write_all(
                template
                    .index
                    .replace("{title}", c.title())
                    .replace("{description}", c.description())
                    .replace("{link}", c.link())
                    .replace("{language}", c.language().unwrap_or_default())
                    .replace("{pub_date}", &time.format(c.pub_date()))
                    .replace("{last_build_date}", &time.format(c.last_build_date()))
                    .replace("{time_generated}", &time.now())
                    .replace("{items}", &{
                        let mut items = String::with_capacity(l);
                        for (n, item) in i.iter().enumerate() {
                            if n > l {
                                break;
                            }
                            items.push_str(
                                &template
                                    .index_item
                                    .replace("{title}", item.title().unwrap_or_default())
                                    .replace(
                                        "{description}",
                                        item.description().unwrap_or_default(),
                                    )
                                    .replace("{link}", item.link().unwrap_or_default())
                                    .replace("{time}", &time.format(item.pub_date())),
                            )
                        }
                        items
                    })
                    .as_bytes(),
            )?,
        }
    }

    Ok(())
}
