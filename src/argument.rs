use clap::Parser;
use std::path::PathBuf;

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
pub struct Argument {
    /// Path to config file
    ///
    /// * see `config/example.toml`
    #[arg(short, long)]
    pub config: PathBuf,
}
