mod connection;
pub mod table;
#[cfg(feature = "transaction")]
mod transaction;

pub use connection::Connection;
#[cfg(feature = "transaction")]
pub use transaction::Transaction;
pub struct Database {
    pool: mysql::Pool,
}

impl Database {
    pub fn pool(
        host: &str,
        port: u16,
        user: &str,
        password: &str,
        database: &str,
    ) -> Result<Self, mysql::Error> {
        Ok(Self {
            pool: mysql::Pool::new(
                format!("mysql://{user}:{password}@{host}:{port}/{database}").as_str(),
            )?,
        })
    }

    pub fn connection(&self) -> Result<Connection, mysql::Error> {
        Connection::create(&self.pool)
    }

    #[cfg(feature = "transaction")]
    pub fn transaction(&self) -> Result<Transaction, mysql::Error> {
        Transaction::create(&self.pool)
    }
}
