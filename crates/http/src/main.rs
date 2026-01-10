#[macro_use]
extern crate rocket;

mod argument;
mod config;
mod feed;
mod global;
mod meta;

use chrono::{DateTime, Utc};
use feed::Feed;
use global::Global;
use meta::Meta;
use mysql::{Database, table::Sort};
use rocket::{
    State,
    http::{ContentType, Status},
    response::content::RawXml,
    serde::Serialize,
};
use rocket_dyn_templates::{Template, context};

#[get("/?<search>&<page>")]
fn index(
    search: Option<&str>,
    page: Option<usize>,
    db: &State<Database>,
    meta: &State<Meta>,
    global: &State<Global>,
) -> Result<Template, Status> {
    #[derive(Serialize)]
    #[serde(crate = "rocket::serde")]
    struct Row {
        content_id: u64,
        link: String,
        time: String,
        title: String,
    }
    let mut conn = db.connection().map_err(|e| {
        error!("Could not connect database: `{e}`");
        Status::InternalServerError
    })?;
    let total = conn
        .contents_total_by_provider_id(global.provider_id, search)
        .map_err(|e| {
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
            rows: conn.contents_by_provider_id(
                        global.provider_id,
                        search,
                        Sort::Desc,
                        Some(global.list_limit)
                    ).map_err(|e| {
                        error!("Could not get contents: `{e}`");
                        Status::InternalServerError
                    })?
                .into_iter()
                .map(|content| {
                    let channel_item = conn.channel_item(content.channel_item_id).unwrap().unwrap();
                    Row {
                        content_id: content.content_id,
                        link: channel_item.link,
                        time: time(channel_item.pub_date).format(&global.format_time).to_string(),
                        title: content.title,
                    }
                })
                .collect::<Vec<Row>>(),
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
    db: &State<Database>,
    meta: &State<Meta>,
    global: &State<Global>,
) -> Result<Template, Status> {
    let mut conn = db.connection().map_err(|e| {
        error!("Could not connect database: `{e}`");
        Status::InternalServerError
    })?;
    match conn.content(content_id).map_err(|e| {
        error!("Could not get content `{content_id}`: `{e}`");
        Status::InternalServerError
    })? {
        Some(content) => {
            let channel_item = conn
                .channel_item(content.channel_item_id)
                .map_err(|e| {
                    error!("Could not get requested channel item: `{e}`");
                    Status::InternalServerError
                })?
                .ok_or_else(|| {
                    error!("Could not find requested channel item");
                    Status::NotFound
                })?;
            Ok(Template::render(
                "info",
                context! {
                    description: content.description,
                    link: channel_item.link,
                    meta: meta.inner(),
                    title: format!("{}{S}{}", content.title, meta.title),
                    name: content.title,
                    time: time(channel_item.pub_date).format(&global.format_time).to_string(),
                },
            ))
        }
        None => Err(Status::NotFound),
    }
}

#[get("/image/<image_id>")]
fn image(image_id: u64, db: &State<Database>) -> Result<(ContentType, Vec<u8>), Status> {
    let mut conn = db.connection().map_err(|e| {
        error!("Could not connect database: `{e}`");
        Status::InternalServerError
    })?;
    match conn.image(image_id).map_err(|e| {
        error!("Could not get content image `{image_id}`: `{e}`");
        Status::InternalServerError
    })? {
        Some(image) => Ok((ContentType::Bytes, image.data)),
        None => Err(Status::NotFound),
    }
}

#[get("/rss?<search>")]
fn rss(
    search: Option<&str>,
    global: &State<Global>,
    meta: &State<Meta>,
    db: &State<Database>,
) -> Result<RawXml<String>, Status> {
    let mut feed = Feed::new(
        &meta.title,
        meta.description.as_deref(),
        1024, // @TODO
    );
    let mut conn = db.connection().map_err(|e| {
        error!("Could not connect database: `{e}`");
        Status::InternalServerError
    })?;
    for content in conn
        .contents_by_provider_id(global.provider_id, search, Sort::Desc, Some(20)) // @TODO
        .map_err(|e| {
            error!("Could not load channel item contents: `{e}`");
            Status::InternalServerError
        })?
    {
        let channel_item = conn
            .channel_item(content.channel_item_id)
            .map_err(|e| {
                error!("Could not get requested channel item: `{e}`");
                Status::InternalServerError
            })?
            .ok_or_else(|| {
                error!("Could not find requested channel item");
                Status::NotFound
            })?;
        feed.push(
            content.channel_item_id,
            time(channel_item.pub_date),
            channel_item.link,
            content.title,
            content.description,
        )
    }
    Ok(RawXml(feed.commit()))
}

#[launch]
fn rocket() -> _ {
    use clap::Parser;
    let argument = argument::Argument::parse();
    let config: config::Config =
        toml::from_str(&std::fs::read_to_string(argument.config).unwrap()).unwrap();
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
        .manage(
            Database::pool(
                &config.mysql.host,
                config.mysql.port,
                &config.mysql.username,
                &config.mysql.password,
                &config.mysql.database,
            )
            .unwrap(),
        )
        .manage(Global {
            format_time: config.format_time,
            list_limit: config.list_limit,
            provider_id: config.provider_id,
        })
        .manage(Meta {
            description: config.description,
            title: config.title,
            version: env!("CARGO_PKG_VERSION").into(),
        })
        .mount("/", routes![index, rss, info, image])
}

const S: &str = " • ";

fn time(timestamp: i64) -> DateTime<Utc> {
    DateTime::<Utc>::from_timestamp(timestamp, 0).unwrap()
}
