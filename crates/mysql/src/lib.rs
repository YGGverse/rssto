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

    /// Get subjects for `rssto-llm` queue
    pub fn contents_queue_for_provider_id(
        &self,
        provider_id: u64,
        sort: Sort,
        limit: Option<usize>,
    ) -> Result<Vec<Content>, Error> {
        self.pool.get_conn()?.exec(
            format!(
                "SELECT `c1`.`content_id`,
                        `c1`.`channel_item_id`,
                        `c1`.`provider_id`,
                        `c1`.`title`,
                        `c1`.`description`
                    FROM `content` AS `c1` WHERE `c1`.`provider_id` IS NULL AND NOT EXISTS (
                        SELECT NULL FROM `content` AS `c2` WHERE `c2`.`channel_item_id` = `c1`.`channel_item_id` AND `c2`.`provider_id` = ? LIMIT 1
                    ) ORDER BY `c1`.`content_id` {sort} LIMIT {}",
                limit.unwrap_or(DEFAULT_LIMIT)
            ),
            (provider_id,),
        )
    }

    pub fn contents_by_channel_item_id_provider_id(
        &self,
        channel_item_id: u64,
        provider_id: Option<u64>,
        limit: Option<usize>,
    ) -> Result<Vec<Content>, Error> {
        self.pool.get_conn()?.exec(
            format!(
                "SELECT `content_id`,
                        `channel_item_id`,
                        `provider_id`,
                        `title`,
                        `description` FROM `content` WHERE `channel_item_id` = ? AND `provider_id` <=> ? LIMIT {}",
                limit.unwrap_or(DEFAULT_LIMIT)
            ),
            (channel_item_id, provider_id),
        )
    }

    pub fn insert_content(
        &self,
        channel_item_id: u64,
        provider_id: Option<u64>,
        title: &str,
        description: &str,
    ) -> Result<u64, Error> {
        let mut c = self.pool.get_conn()?;
        c.exec_drop(
            "INSERT INTO `content` SET `channel_item_id` = ?, `provider_id` = ?, `title` = ?, `description` = ?",
            (channel_item_id, provider_id, title, description ),
        )?;
        Ok(c.last_insert_id())
    }

    pub fn provider_by_name(&self, name: &str) -> Result<Option<Provider>, Error> {
        self.pool.get_conn()?.exec_first(
            "SELECT `provider_id`,
                    `name`
                    FROM `provider` WHERE `name` = ?",
            (name,),
        )
    }

    pub fn insert_provider(&self, name: &str) -> Result<u64, Error> {
        let mut c = self.pool.get_conn()?;
        c.exec_drop("INSERT INTO `provider` SET `name` = ?", (name,))?;
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
    pub provider_id: Option<u64>,
    pub title: String,
    pub description: String,
}

#[derive(Debug, PartialEq, Eq, FromRow)]
pub struct Provider {
    pub provider_id: u64,
    pub name: String,
}

pub enum Sort {
    Asc,
    Desc,
}

impl std::fmt::Display for Sort {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Self::Asc => write!(f, "ASC"),
            Self::Desc => write!(f, "DESC"),
        }
    }
}

/// Shared search logic
fn like(value: Option<&str>) -> String {
    value.map_or("%".into(), |k| format!("{k}%"))
}

const DEFAULT_LIMIT: usize = 100;
