use clap::Parser;
use std::net::{IpAddr, Ipv4Addr};

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
pub struct Config {
    /// Server name
    #[arg(long, default_value_t = String::from("rssto"))]
    pub title: String,

    /// Server description
    #[arg(long)]
    pub description: Option<String>,

    /// Format timestamps (on the web view)
    ///
    /// * tip: escape with `%%d/%%m/%%Y %%H:%%M` in the CLI/bash argument
    #[arg(long, default_value_t = String::from("%d/%m/%Y %H:%M"))]
    pub format_time: String,

    /// Provider ID (`provider` table)
    /// * None for the original content
    #[arg(long, short)]
    pub provider_id: Option<u64>,

    /// Default listing limit
    #[arg(long, default_value_t = 20)]
    pub list_limit: usize,

    /// Bind server on given host
    #[arg(long, default_value_t = IpAddr::V4(Ipv4Addr::LOCALHOST))]
    pub host: IpAddr,

    /// Bind server on given port
    #[arg(long, default_value_t = 8000)]
    pub port: u16,

    /// Configure instance in the debug mode
    #[arg(long, default_value_t = false)]
    pub debug: bool,

    // Database
    #[arg(long, default_value_t = String::from("localhost"))]
    pub mysql_host: String,
    #[arg(long, default_value_t = 3306)]
    pub mysql_port: u16,
    #[arg(long)]
    pub mysql_username: String,
    #[arg(long)]
    pub mysql_password: String,
    #[arg(long)]
    pub mysql_database: String,
}
