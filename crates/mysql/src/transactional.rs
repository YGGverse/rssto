use crate::table::*;
use mysql::{Error, Pool, Transaction, TxOpts, prelude::Queryable};

/// Safe, optimized read/write operations
/// mostly required by the `rssto-crawler` and `rssto-llm`
/// * all members implementation requires `commit` action
pub struct Transactional {
    tx: Transaction<'static>,
}

impl Transactional {
    pub fn connect(
        host: &str,
        port: u16,
        user: &str,
        password: &str,
        database: &str,
    ) -> Result<Self, Error> {
        Ok(Self {
            tx: Pool::new(format!("mysql://{user}:{password}@{host}:{port}/{database}").as_str())?
                .start_transaction(TxOpts::default())?,
        })
    }

    pub fn commit(self) -> Result<(), Error> {
        self.tx.commit()
    }

    pub fn channel_id_by_url(&mut self, url: &str) -> Result<Option<u64>, Error> {
        self.tx.exec_first(
            "SELECT `channel_id` FROM `channel` WHERE `url` = ? LIMIT 1",
            (url,),
        )
    }

    pub fn insert_channel(&mut self, url: &str) -> Result<u64, Error> {
        self.tx
            .exec_drop("INSERT INTO `channel` SET `url` = ?", (url,))?;
        Ok(self.tx.last_insert_id().unwrap())
    }

    pub fn channel_items_total_by_channel_id_guid(
        &mut self,
        channel_id: u64,
        guid: &str,
    ) -> Result<usize, Error> {
        Ok(self
            .tx
            .exec_first(
                "SELECT COUNT(*) FROM `channel_item` WHERE `channel_id` = ? AND `guid` = ?",
                (channel_id, guid),
            )?
            .unwrap_or(0))
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
        self.tx.exec_drop(
            "INSERT INTO `channel_item` SET `channel_id` = ?,
                                            `pub_date` = ?,
                                            `guid` = ?,
                                            `link` = ?,
                                            `title` = ?,
                                            `description` = ?",
            (channel_id, pub_date, guid, link, title, description),
        )?;
        Ok(self.tx.last_insert_id().unwrap())
    }

    pub fn contents_queue_for_provider_id(
        &mut self,
        provider_id: u64,
    ) -> Result<Vec<Content>, Error> {
        self.tx.exec(
            "SELECT `c1`.`content_id`,
                    `c1`.`channel_item_id`,
                    `c1`.`provider_id`,
                    `c1`.`title`,
                    `c1`.`description`
            FROM `content` AS `c1` WHERE `c1`.`provider_id` IS NULL AND NOT EXISTS (
                SELECT NULL FROM  `content` AS `c2`
                            WHERE `c2`.`channel_item_id` = `c1`.`channel_item_id`
                              AND `c2`.`provider_id` = ? LIMIT 1
            )",
            (provider_id,),
        )
    }

    pub fn insert_content(
        &mut self,
        channel_item_id: u64,
        provider_id: Option<u64>,
        title: &str,
        description: &str,
    ) -> Result<u64, Error> {
        self.tx.exec_drop(
            "INSERT INTO `content` SET  `channel_item_id` = ?,
                                        `provider_id` = ?,
                                        `title` = ?,
                                        `description` = ?",
            (channel_item_id, provider_id, title, description),
        )?;
        Ok(self.tx.last_insert_id().unwrap())
    }

    pub fn insert_content_image(&mut self, content_id: u64, image_id: u64) -> Result<u64, Error> {
        self.tx.exec_drop(
            "INSERT INTO `content_image` SET `content_id` = ?, `image_id` = ?",
            (content_id, image_id),
        )?;
        Ok(self.tx.last_insert_id().unwrap())
    }

    pub fn images_total_by_source(&mut self, source: &str) -> Result<usize, Error> {
        Ok(self
            .tx
            .exec_first("SELECT COUNT(*) FROM `image` WHERE `source` = ?", (source,))?
            .unwrap_or(0))
    }

    pub fn insert_image(&mut self, source: &str, data: &[u8]) -> Result<u64, Error> {
        self.tx.exec_drop(
            "INSERT INTO `image` SET `source` = ?, `data` = ?",
            (source, data),
        )?;
        Ok(self.tx.last_insert_id().unwrap())
    }

    pub fn provider_id_by_name(&mut self, name: &str) -> Result<Option<u64>, Error> {
        self.tx.exec_first(
            "SELECT `provider_id` FROM `provider` WHERE `name` = ?",
            (name,),
        )
    }

    pub fn insert_provider(&mut self, name: &str) -> Result<u64, Error> {
        self.tx
            .exec_drop("INSERT INTO `provider` SET `name` = ?", (name,))?;
        Ok(self.tx.last_insert_id().unwrap())
    }
}
