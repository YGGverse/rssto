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
                    `link`,
                    `title`,
                    `description` FROM `channel_item` WHERE `channel_item_id` = ?",
            (channel_item_id,),
        )
    }

    pub fn content(&mut self, content_id: u64) -> Result<Option<Content>, Error> {
        self.conn.exec_first(
            "SELECT `content_id`,
                    `channel_item_id`,
                    `provider_id`,
                    `title`,
                    `description` FROM `content` WHERE `content_id` = ?",
            (content_id,),
        )
    }

    pub fn contents_total_by_provider_id(
        &mut self,
        provider_id: Option<u64>,
        keyword: Option<&str>,
    ) -> Result<usize, Error> {
        let total: Option<usize> = self.conn.exec_first(
            "SELECT COUNT(*) FROM `content` WHERE `provider_id` <=> ? AND `title` LIKE ?",
            (provider_id, like(keyword)),
        )?;
        Ok(total.unwrap_or(0))
    }

    pub fn contents_by_provider_id(
        &mut self,
        provider_id: Option<u64>,
        keyword: Option<&str>,
        sort: Sort,
        limit: Option<usize>,
    ) -> Result<Vec<Content>, Error> {
        self.conn.exec(format!(
            "SELECT `content_id`,
                    `channel_item_id`,
                    `provider_id`,
                    `title`,
                    `description` FROM `content` WHERE `provider_id` <=> ? AND `title` LIKE ? ORDER BY `content_id` {sort} LIMIT {}",
            limit.unwrap_or(DEFAULT_LIMIT)
        ),
        (provider_id, like(keyword), ))
    }

    pub fn content_image(&mut self, content_image_id: u64) -> Result<Option<ContentImage>, Error> {
        self.conn.exec_first(
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

    pub fn image(&mut self, image_id: u64) -> Result<Option<Image>, Error> {
        self.conn.exec_first(
            "SELECT `image_id`,
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

/// Shared search logic
fn like(value: Option<&str>) -> String {
    value.map_or("%".into(), |k| format!("{k}%"))
}

const DEFAULT_LIMIT: usize = 100;
