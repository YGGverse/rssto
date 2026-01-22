use serde::Deserialize;
use std::net::IpAddr;

#[derive(Debug, Deserialize)]
pub struct Mysql {
    pub database: String,
    pub host: String,
    pub password: String,
    pub port: u16,
    pub username: String,
}

#[derive(Debug, Deserialize)]
pub struct Config {
    pub mysql: Mysql,
    pub title: String,
    pub description: Option<String>,
    pub format_time: String,
    pub provider_id: Option<u64>,
    pub list_limit: usize,
    pub host: IpAddr,
    pub port: u16,
    pub debug: bool,
}
