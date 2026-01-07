use mysql::{Error, PooledConn, prelude::Queryable};

pub struct Mysql {
    connection: PooledConn,
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
            connection: mysql::Pool::new(
                format!("mysql://{user}:{password}@{host}:{port}/{database}").as_str(),
            )?
            .get_conn()?,
        })
    }

    pub fn channels_by_url(
        &mut self,
        url: &str,
        limit: Option<usize>,
    ) -> Result<Vec<Channel>, Error> {
        self.connection.exec_map(
            format!(
                "SELECT `channel_id`, `url` FROM `channel` WHERE `url` = ? LIMIT {}",
                limit.unwrap_or(DEFAULT_LIMIT)
            ),
            (url,),
            |(channel_id, url)| Channel { channel_id, url },
        )
    }

    pub fn insert_channel(&mut self, url: &str) -> Result<u64, Error> {
        self.connection
            .exec_drop("INSERT INTO `channel` SET `url` = ?", (url,))?;

        Ok(self.connection.last_insert_id())
    }

    pub fn channel_items_by_channel_id_guid(
        &mut self,
        channel_id: u64,
        guid: &str,
        limit: Option<usize>,
    ) -> Result<Vec<ChannelItem>, Error> {
        self.connection.exec_map(
            format!(
                "SELECT `channel_item_id`, `channel_id`, `pub_date`, `guid`, `link`, `title`, `description` FROM `channel_item` WHERE `channel_id` = ? AND `guid` = ? LIMIT {}",
                limit.unwrap_or(DEFAULT_LIMIT)),
            (
                channel_id,
                guid
            ),
            |(
                channel_item_id,
                channel_id,
                pub_date,
                guid,
                link,
                title,
                description
            )|
            ChannelItem {
                channel_item_id,
                channel_id,
                pub_date,
                guid,
                link,
                title,
                description
            },
        )
    }

    pub fn insert_channel_item(
        &mut self,
        channel_id: u64,
        pub_date: i64,
        guid: &str,
        link: &str,
        title: Option<&str>,
        description: Option<&str>,
    ) -> Result<u64, Error> {
        self.connection.exec_drop(
            "INSERT INTO `channel_item` SET `channel_id` = ?, `pub_date` = ?, `guid` = ?, `link` = ?, `title` = ?, `description` = ?",
            (channel_id, pub_date, guid, link, title, description),
        )?;
        Ok(self.connection.last_insert_id())
    }

    pub fn insert_content(
        &mut self,
        channel_item_id: u64,
        source_id: Option<u64>,
        title: &str,
        description: &str,
    ) -> Result<(), Error> {
        self.connection.exec_drop(
            "INSERT INTO `content` SET `channel_item_id` = ?, `source_id` = ?, `title` = ?, `description` = ?",
            (channel_item_id, source_id, title, description ),
        )
    }
}

#[derive(Debug, PartialEq, Eq)]
pub struct Channel {
    pub channel_id: u64,
    pub url: String,
}

#[derive(Debug, PartialEq, Eq)]
pub struct ChannelItem {
    pub channel_item_id: u64,
    pub channel_id: u64,
    pub pub_date: i64,
    pub guid: String,
    pub link: String,
    pub title: Option<String>,
    pub description: Option<String>,
}

#[derive(Debug, PartialEq, Eq)]
pub struct Content {
    pub channel_item_id: u64,
    /// None if the original `title` and `description` values
    /// parsed from the channel item on crawl
    pub source_id: Option<u64>,
    pub title: String,
    pub description: String,
}

const DEFAULT_LIMIT: usize = 100;
