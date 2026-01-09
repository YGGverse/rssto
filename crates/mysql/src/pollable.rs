pub mod sort;

pub use sort::Sort;

use crate::table::*;
use mysql::{Error, Pool, prelude::Queryable};

/// Safe, read-only operations used in client apps like `rssto-http`
pub struct Pollable {
    pool: Pool,
}

impl Pollable {
    pub fn connect(
        host: &str,
        port: u16,
        user: &str,
        password: &str,
        database: &str,
    ) -> Result<Self, Error> {
        Ok(Self {
            pool: mysql::Pool::new(
                format!("mysql://{user}:{password}@{host}:{port}/{database}").as_str(),
            )?,
        })
    }

    pub fn channel_item(&self, channel_item_id: u64) -> Result<Option<ChannelItem>, Error> {
        self.pool.get_conn()?.exec_first(
            "SELECT `channel_item_id`,
                    `channel_id`,
                    `pub_date`,
                    `guid`,
                    `link`,
                    `title`,
                    `description` FROM `channel_item` WHERE `channel_item_id` = ?",
            (channel_item_id,),
        )
    }

    pub fn content(&self, content_id: u64) -> Result<Option<Content>, Error> {
        self.pool.get_conn()?.exec_first(
            "SELECT `content_id`,
                    `channel_item_id`,
                    `provider_id`,
                    `title`,
                    `description` FROM `content` WHERE `content_id` = ?",
            (content_id,),
        )
    }

    pub fn contents_total_by_provider_id(
        &self,
        provider_id: Option<u64>,
        keyword: Option<&str>,
    ) -> Result<usize, Error> {
        let total: Option<usize> = self.pool.get_conn()?.exec_first(
            "SELECT COUNT(*) FROM `content` WHERE `provider_id` <=> ? AND `title` LIKE ?",
            (provider_id, like(keyword)),
        )?;
        Ok(total.unwrap_or(0))
    }

    pub fn contents_by_provider_id(
        &self,
        provider_id: Option<u64>,
        keyword: Option<&str>,
        sort: Sort,
        limit: Option<usize>,
    ) -> Result<Vec<Content>, Error> {
        self.pool.get_conn()?.exec(format!(
            "SELECT `content_id`,
                    `channel_item_id`,
                    `provider_id`,
                    `title`,
                    `description` FROM `content` WHERE `provider_id` <=> ? AND `title` LIKE ? ORDER BY `content_id` {sort} LIMIT {}",
            limit.unwrap_or(DEFAULT_LIMIT)
        ),
        (provider_id, like(keyword), ))
    }

    pub fn content_image(&self, content_image_id: u64) -> Result<Option<ContentImage>, Error> {
        self.pool.get_conn()?.exec_first(
            "SELECT `content_image_id`,
                    `content_id`,
                    `image_id`,
                    `data`,
                    `source` FROM `content_image`
                             JOIN  `image` ON (`image`.`image_id` = `content_image`.`image_id`)
                             WHERE `content_image_id` = ? LIMIT 1",
            (content_image_id,),
        )
    }

    pub fn images(&self, limit: Option<usize>) -> Result<Vec<Image>, Error> {
        self.pool.get_conn()?.query(format!(
            "SELECT `image_id`, `source`, `data` FROM `image` LIMIT {}",
            limit.unwrap_or(DEFAULT_LIMIT)
        ))
    }

    pub fn insert_provider(&self, name: &str) -> Result<u64, Error> {
        let mut c = self.pool.get_conn()?;
        c.exec_drop("INSERT INTO `provider` SET `name` = ?", (name,))?;
        Ok(c.last_insert_id())
    }
}

/// Shared search logic
fn like(value: Option<&str>) -> String {
    value.map_or("%".into(), |k| format!("{k}%"))
}

const DEFAULT_LIMIT: usize = 100;
