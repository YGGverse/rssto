use crate::table::*;
use mysql::{Error, Pool, PooledConn, prelude::Queryable};

/// Safe, read-only operations used in client apps like `rssto-http`
pub struct Connection {
    conn: PooledConn,
}

impl Connection {
    pub fn create(pool: &Pool) -> Result<Self, Error> {
        Ok(Self {
            conn: pool.get_conn()?,
        })
    }

    pub fn channel_item(&mut self, channel_item_id: u64) -> Result<Option<ChannelItem>, Error> {
        self.conn.exec_first(
            "SELECT `channel_item_id`,
                    `channel_id`,
                    `pub_date`,
                    `guid`,
                    `link` FROM `channel_item` WHERE `channel_item_id` = ?",
            (channel_item_id,),
        )
    }

    pub fn channel_item_content(
        &mut self,
        channel_item_content_id: u64,
    ) -> Result<Option<ChannelItemContent>, Error> {
        self.conn.exec_first(
            "SELECT `channel_item_content_id`,
                    `channel_item_id`
                    FROM `channel_item_content` WHERE `channel_item_content_id` = ?",
            (channel_item_content_id,),
        )
    }

    pub fn channel_item_content_description(
        &mut self,
        channel_item_content_description_id: u64,
    ) -> Result<Option<ChannelItemContentDescription>, Error> {
        self.conn.exec_first(
            "SELECT `channel_item_content_description_id`,
                    `channel_item_content_id`,
                    `provider_id`,
                    `title`,
                    `description` FROM `channel_item_content_description`
                                  WHERE `channel_item_content_description_id` = ?",
            (channel_item_content_description_id,),
        )
    }

    pub fn channel_item_content_descriptions_total_by_provider_id(
        &mut self,
        provider_id: Option<u64>,
        keyword: Option<&str>,
    ) -> Result<usize, Error> {
        let total: Option<usize> = match keyword {
            Some(k) => self.conn.exec_first(
                "SELECT COUNT(*) FROM `channel_item_content_description`
                                 WHERE `provider_id` <=> ? AND `title` LIKE '%?%'",
                (provider_id, k),
            )?,
            None => self.conn.exec_first(
                "SELECT COUNT(*) FROM `channel_item_content_description`
                                 WHERE `provider_id` <=> ?",
                (provider_id,),
            )?,
        };

        Ok(total.unwrap_or(0))
    }

    pub fn channel_item_content_descriptions_by_provider_id(
        &mut self,
        provider_id: Option<u64>,
        keyword: Option<&str>,
        sort: Sort,
        start: Option<usize>,
        limit: Option<usize>,
    ) -> Result<Vec<ChannelItemContentDescription>, Error> {
        match keyword {
            Some(k) => self.conn.exec(
                format!(
                    "SELECT `channel_item_content_description_id`,
                        `channel_item_content_id`,
                        `provider_id`,
                        `title`,
                        `description`
                    FROM  `channel_item_content_description`
                    WHERE `provider_id` <=> ? AND `title` LIKE '%?%'
                    ORDER BY `channel_item_content_description_id` {sort} LIMIT {},{}",
                    start.unwrap_or(0),
                    limit.unwrap_or(DEFAULT_LIMIT)
                ),
                (provider_id, k),
            ),
            None => self.conn.exec(
                format!(
                    "SELECT `channel_item_content_description_id`,
                        `channel_item_content_id`,
                        `provider_id`,
                        `title`,
                        `description`
                    FROM  `channel_item_content_description`
                    WHERE `provider_id` <=> ?
                    ORDER BY `channel_item_content_description_id` {sort} LIMIT {},{}",
                    start.unwrap_or(0),
                    limit.unwrap_or(DEFAULT_LIMIT)
                ),
                (provider_id,),
            ),
        }
    }

    pub fn image(&mut self, image_id: u64) -> Result<Option<Image>, Error> {
        self.conn.exec_first(
            "SELECT `image_id`,
                    `provider_id`,
                    `sha256`,
                    `src`,
                    `url`,
                    `data` FROM `image` WHERE `image_id` = ?",
            (image_id,),
        )
    }

    pub fn provider_id_by_name(&mut self, name: &str) -> Result<Option<u64>, Error> {
        self.conn.exec_first(
            "SELECT `provider_id` FROM `provider` WHERE `name` = ?",
            (name,),
        )
    }

    pub fn insert_provider(&mut self, name: &str) -> Result<u64, Error> {
        self.conn
            .exec_drop("INSERT INTO `provider` SET `name` = ?", (name,))?;
        Ok(self.conn.last_insert_id())
    }
}

const DEFAULT_LIMIT: usize = 100;
