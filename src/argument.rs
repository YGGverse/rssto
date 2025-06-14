use clap::Parser;

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
pub struct Argument {
    /// Show output (`d` - debug, `e` - error, `i` - info)
    #[arg(short, long, default_value_t = String::from("ei"))]
    pub debug: String,

    /// Export formats (`html`,`md`,etc.)
    #[arg(short, long, default_values_t = [String::from("html")])]
    pub format: Vec<String>,

    /// Limit channel items (unlimited by default)
    #[arg(short, long)]
    pub limit: Option<usize>,

    /// RSS feed URL(s)
    #[arg(short, long)]
    pub source: Vec<String>,

    /// Destination directory
    #[arg(long)]
    pub target: Vec<String>,

    /// Path to template directory
    #[arg(long, default_value_t = String::from("template"))]
    pub template: String,

    /// Use custom time format
    #[arg(long, default_value_t = String::from("%Y/%m/%d %H:%M:%S %z"))]
    pub time_format: String,

    /// Update timeout in seconds
    #[arg(short, long, default_value_t = 60)]
    pub update: u64,
}
