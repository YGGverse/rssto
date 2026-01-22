use serde::Deserialize;
use std::net::IpAddr;

#[derive(Debug, Deserialize)]
pub struct Mysql {
    pub database: String,
    pub host: IpAddr,
    pub password: String,
    pub port: u16,
    pub username: String,
}

#[derive(Debug, Deserialize)]
pub struct Llm {
    pub scheme: String,
    pub host: IpAddr,
    pub port: u16,
    pub model: String,
    pub message: String,
}

#[derive(Debug, Deserialize)]
pub struct Config {
    pub mysql: Mysql,
    pub llm: Llm,
    pub update: Option<u64>,
}
