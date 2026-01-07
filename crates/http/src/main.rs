#[macro_use]
extern crate rocket;

mod config;
mod feed;
mod global;
mod meta;

use chrono::{DateTime, Utc};
use config::Config;
use feed::Feed;
use global::Global;
use meta::Meta;
use mysql::Mysql;
use rocket::{State, http::Status, response::content::RawXml, serde::Serialize};
use rocket_dyn_templates::{Template, context};

#[get("/?<search>&<page>")]
fn index(
    search: Option<&str>,
    page: Option<usize>,
    db: &State<Mysql>,
    meta: &State<Meta>,
    global: &State<Global>,
) -> Result<Template, Status> {
    #[derive(Serialize)]
    #[serde(crate = "rocket::serde")]
    struct Content {
        content_id: u64,
        description: String,
        link: String,
        time: String,
        title: String,
    }
    let total = db.contents_total().map_err(|e| {
        error!("Could not get contents total: `{e}`");
        Status::InternalServerError
    })?;
    Ok(Template::render(
        "index",
        context! {
            title: {
                let mut t = String::new();
                if let Some(q) = search && !q.is_empty() {
                    t.push_str(q);
                    t.push_str(S);
                    t.push_str("Search");
                    t.push_str(S)
                }
                if let Some(p) = page && p > 1 {
                    t.push_str(&format!("Page {p}"));
                    t.push_str(S)
                }
                t.push_str(&meta.title);
                if let Some(ref description) = meta.description
                        && page.is_none_or(|p| p == 1) && search.is_none_or(|q| q.is_empty()) {
                    t.push_str(S);
                    t.push_str(description)
                }
                t
            },
            meta: meta.inner(),
            back: page.map(|p| uri!(index(search, if p > 2 { Some(p - 1) } else { None }))),
            next: if page.unwrap_or(1) * global.list_limit >= total { None }
                    else { Some(uri!(index(search, Some(page.map_or(2, |p| p + 1))))) },
            rows: db.contents(Some(global.list_limit)).map_err(|e| {
                error!("Could not get contents: `{e}`");
                Status::InternalServerError
            })?
                .into_iter()
                .map(|c| {
                    let channel_item = db.channel_item(c.channel_item_id).unwrap().unwrap();
                    Content {
                        content_id: c.content_id,
                        description: c.description,
                        link: channel_item.link,
                        time: time(channel_item.pub_date).format(&global.format_time).to_string(),
                        title: c.title,
                    }
                })
                .collect::<Vec<Content>>(),
            page: page.unwrap_or(1),
            pages: (total as f64 / global.list_limit as f64).ceil(),
            total,
            search
        },
    ))
}

#[get("/<content_id>")]
fn info(
    content_id: u64,
    db: &State<Mysql>,
    meta: &State<Meta>,
    global: &State<Global>,
) -> Result<Template, Status> {
    match db.content(content_id).map_err(|e| {
        error!("Could not get content `{content_id}`: `{e}`");
        Status::InternalServerError
    })? {
        Some(c) => {
            let i = db.channel_item(c.channel_item_id).unwrap().unwrap();
            Ok(Template::render(
                "info",
                context! {
                    title: format!("{}{S}{}", c.title, meta.title),
                    description: c.description,
                    link: i.link,
                    time: time(i.pub_date).format(&global.format_time).to_string(),
                },
            ))
        }
        None => Err(Status::NotFound),
    }
}

#[get("/rss")]
fn rss(meta: &State<Meta>, db: &State<Mysql>) -> Result<RawXml<String>, Status> {
    let mut f = Feed::new(
        &meta.title,
        meta.description.as_deref(),
        1024, // @TODO
    );
    for c in db
        .contents(Some(20)) // @TODO
        .map_err(|e| {
            error!("Could not load channel item contents: `{e}`");
            Status::InternalServerError
        })?
    {
        let channel_item = db.channel_item(c.channel_item_id).unwrap().unwrap();
        f.push(
            c.channel_item_id,
            time(channel_item.pub_date),
            channel_item.link,
            c.title,
            c.description,
        )
    }
    Ok(RawXml(f.commit()))
}

#[launch]
fn rocket() -> _ {
    use clap::Parser;
    let config = Config::parse();
    rocket::build()
        .attach(Template::fairing())
        .configure(rocket::Config {
            port: config.port,
            address: config.host,
            ..if config.debug {
                rocket::Config::debug_default()
            } else {
                rocket::Config::release_default()
            }
        })
        .manage(Mysql::connect(
            &config.mysql_host,
            config.mysql_port,
            &config.mysql_user,
            &config.mysql_password,
            &config.mysql_database,
        ))
        .manage(Global {
            format_time: config.format_time,
            list_limit: config.list_limit,
        })
        .manage(Meta {
            description: config.description,
            title: config.title,
            version: env!("CARGO_PKG_VERSION").into(),
        })
        .mount("/", routes![index, rss, info])
}

const S: &str = " • ";

fn time(timestamp: i64) -> DateTime<Utc> {
    DateTime::<Utc>::from_timestamp(timestamp, 0).unwrap()
}
