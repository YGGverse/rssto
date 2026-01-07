use mysql::{
    Error, Pool,
    prelude::{FromRow, Queryable},
};

pub struct Mysql {
    pool: Pool,
}

impl Mysql {
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

    pub fn channels_by_url(&self, url: &str, limit: Option<usize>) -> Result<Vec<Channel>, Error> {
        self.pool.get_conn()?.exec_map(
            format!(
                "SELECT `channel_id`, `url` FROM `channel` WHERE `url` = ? LIMIT {}",
                limit.unwrap_or(DEFAULT_LIMIT)
            ),
            (url,),
            |(channel_id, url)| Channel { channel_id, url },
        )
    }

    pub fn insert_channel(&self, url: &str) -> Result<u64, Error> {
        let mut c = self.pool.get_conn()?;
        c.exec_drop("INSERT INTO `channel` SET `url` = ?", (url,))?;
        Ok(c.last_insert_id())
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

    pub fn channel_items_by_channel_id_guid(
        &self,
        channel_id: u64,
        guid: &str,
        limit: Option<usize>,
    ) -> Result<Vec<ChannelItem>, Error> {
        self.pool.get_conn()?.exec(
            format!(
                "SELECT `channel_item_id`,
                        `channel_id`,
                        `pub_date`,
                        `guid`,
                        `link`,
                        `title`,
                        `description` FROM `channel_item` WHERE `channel_id` = ? AND `guid` = ? LIMIT {}",
                limit.unwrap_or(DEFAULT_LIMIT)
            ),
            (channel_id, guid),
        )
    }

    pub fn insert_channel_item(
        &self,
        channel_id: u64,
        pub_date: i64,
        guid: &str,
        link: &str,
        title: Option<&str>,
        description: Option<&str>,
    ) -> Result<u64, Error> {
        let mut c = self.pool.get_conn()?;
        c.exec_drop(
            "INSERT INTO `channel_item` SET `channel_id` = ?,
                                            `pub_date` = ?,
                                            `guid` = ?,
                                            `link` = ?,
                                            `title` = ?,
                                            `description` = ?",
            (channel_id, pub_date, guid, link, title, description),
        )?;
        Ok(c.last_insert_id())
    }

    pub fn content(&self, content_id: u64) -> Result<Option<Content>, Error> {
        self.pool.get_conn()?.exec_first(
            "SELECT `content_id`,
                    `channel_item_id`,
                    `source_id`,
                    `title`,
                    `description` FROM `content` WHERE `content_id` = ?",
            (content_id,),
        )
    }

    pub fn contents_total(&self) -> Result<usize, Error> {
        let total: Option<usize> = self
            .pool
            .get_conn()?
            .query_first("SELECT COUNT(*) FROM `content`")?;
        Ok(total.unwrap_or(0))
    }

    pub fn contents(&self, limit: Option<usize>) -> Result<Vec<Content>, Error> {
        self.pool.get_conn()?.query(format!(
            "SELECT `content_id`,
                    `channel_item_id`,
                    `source_id`,
                    `title`,
                    `description` FROM `content` LIMIT {}",
            limit.unwrap_or(DEFAULT_LIMIT)
        ))
    }

    pub fn contents_by_channel_item_id_source_id(
        &self,
        channel_item_id: u64,
        source_id: Option<u64>,
        limit: Option<usize>,
    ) -> Result<Vec<Content>, Error> {
        self.pool.get_conn()?.exec(
            format!(
                "SELECT `content_id`,
                        `channel_item_id`,
                        `source_id`,
                        `title`,
                        `description` FROM `content` WHERE `channel_item_id` = ? AND `source_id` = ? LIMIT {}",
                limit.unwrap_or(DEFAULT_LIMIT)
            ),
            (channel_item_id, source_id),
        )
    }

    pub fn insert_content(
        &self,
        channel_item_id: u64,
        source_id: Option<u64>,
        title: String,
        description: String,
    ) -> Result<u64, Error> {
        let mut c = self.pool.get_conn()?;
        c.exec_drop(
            "INSERT INTO `content` SET `channel_item_id` = ?, `source_id` = ?, `title` = ?, `description` = ?",
            (channel_item_id, source_id, title, description ),
        )?;
        Ok(c.last_insert_id())
    }
}

#[derive(Debug, PartialEq, Eq, FromRow)]
pub struct Channel {
    pub channel_id: u64,
    pub url: String,
}

#[derive(Debug, PartialEq, Eq, FromRow)]
pub struct ChannelItem {
    pub channel_item_id: u64,
    pub channel_id: u64,
    pub pub_date: i64,
    pub guid: String,
    pub link: String,
    pub title: Option<String>,
    pub description: Option<String>,
}

#[derive(Debug, PartialEq, Eq, FromRow)]
pub struct Content {
    pub content_id: u64,
    pub channel_item_id: u64,
    /// None if the original `title` and `description` values
    /// parsed from the channel item on crawl
    pub source_id: Option<u64>,
    pub title: String,
    pub description: String,
}

const DEFAULT_LIMIT: usize = 100;
