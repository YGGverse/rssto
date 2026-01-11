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
        channel_item_content_description_id: u64,
        link: String,
        time: String,
        title: String,
    }
    let mut conn = db.connection().map_err(|e| {
        error!("Could not connect database: `{e}`");
        Status::InternalServerError
    })?;
    let total = conn
        .channel_item_content_descriptions_total_by_provider_id(global.provider_id, search)
        .map_err(|e| {
            error!("Could not get contents total: `{e}`");
            Status::InternalServerError
        })?;
    Ok(Template::render(
        "index",
        context! {
            title: {
                let mut t = String::with_capacity(9);
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
            rows: conn.channel_item_content_descriptions_by_provider_id(
                        global.provider_id,
                        search,
                        Sort::Desc,
                        page.map(|p| if p > 1 { p - 1 } else { 1 } * global.list_limit),
                        Some(global.list_limit)
                    ).map_err(|e| {
                        error!("Could not get contents: `{e}`");
                        Status::InternalServerError
                    })?
                .into_iter()
                .map(|channel_item_content_description| {
                    let channel_item = conn.channel_item(
                        channel_item_content_description.channel_item_content_id
                    ).unwrap().unwrap();
                    Row {
                        channel_item_content_description_id:
                            channel_item_content_description.channel_item_content_description_id,
                        link: channel_item.link,
                        time: time(channel_item.pub_date).format(&global.format_time).to_string(),
                        title: channel_item_content_description.title.unwrap_or_default(), // @TODO handle
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

#[get("/<channel_item_content_description_id>")]
fn info(
    channel_item_content_description_id: u64,
    db: &State<Database>,
    meta: &State<Meta>,
    global: &State<Global>,
) -> Result<Template, Status> {
    let mut conn = db.connection().map_err(|e| {
        error!("Could not connect database: `{e}`");
        Status::InternalServerError
    })?;
    match conn.channel_item_content_description(channel_item_content_description_id).map_err(|e| {
        error!("Could not get `channel_item_content_description_id` {channel_item_content_description_id}: `{e}`");
        Status::InternalServerError
    })? {
        Some(channel_item_content_description) => {
            let channel_item_content = conn
                .channel_item_content(channel_item_content_description.channel_item_content_id)
                .map_err(|e| {
                    error!(
                        "Could not get requested `channel_item_content` #{}: `{e}`",
                        channel_item_content_description.channel_item_content_id
                    );
                    Status::InternalServerError
                })?
                .ok_or_else(|| {
                    error!(
                        "Could not find requested `channel_item_content` #{}",
                        channel_item_content_description.channel_item_content_id
                    );
                    Status::NotFound
                })?;
            let channel_item = conn
                .channel_item(channel_item_content.channel_item_id)
                .map_err(|e| {
                    error!(
                        "Could not get requested `channel_item` #{}: `{e}`",
                        channel_item_content.channel_item_id
                    );
                    Status::InternalServerError
                })?
                .ok_or_else(|| {
                    error!(
                        "Could not find requested `channel_item` #{}",
                        channel_item_content.channel_item_id
                    );
                    Status::NotFound
                })?;
            let title = channel_item_content_description.title.unwrap_or_default(); // @TODO handle
            Ok(Template::render(
                "info",
                context! {
                    description: channel_item_content_description.description,
                    link: channel_item.link,
                    meta: meta.inner(),
                    title: format!("{title}{S}{}", meta.title),
                    name: title,
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
    for channel_item_content_description in conn
        .channel_item_content_descriptions_by_provider_id(
            global.provider_id,
            search,
            Sort::Desc,
            None,
            Some(global.list_limit),
        )
        .map_err(|e| {
            error!(
                "Could not load `channel_item_content_description` for `provider` #{:?}: `{e}`",
                global.provider_id
            );
            Status::InternalServerError
        })?
    {
        let channel_item_content = conn
            .channel_item_content(channel_item_content_description.channel_item_content_id)
            .map_err(|e| {
                error!(
                    "Could not get requested `channel_item_content` #{}: `{e}`",
                    channel_item_content_description.channel_item_content_id
                );
                Status::InternalServerError
            })?
            .ok_or_else(|| {
                error!(
                    "Could not find requested `channel_item_content` #{}",
                    channel_item_content_description.channel_item_content_id
                );
                Status::NotFound
            })?;
        let channel_item = conn
            .channel_item(channel_item_content.channel_item_id)
            .map_err(|e| {
                error!(
                    "Could not get requested `channel_item` #{}: `{e}`",
                    channel_item_content.channel_item_id
                );
                Status::InternalServerError
            })?
            .ok_or_else(|| {
                error!(
                    "Could not find requested `channel_item` #{}",
                    channel_item_content.channel_item_id
                );
                Status::NotFound
            })?;
        feed.push(
            channel_item_content_description.channel_item_content_description_id,
            time(channel_item.pub_date),
            channel_item.link,
            channel_item_content_description.title.unwrap_or_default(), // @TODO handle
            channel_item_content_description
                .description
                .unwrap_or_default(), // @TODO handle
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
