use clap::Parser;

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
pub struct Argument {
    // LLM
    #[arg(long, default_value_t = String::from("http"))]
    pub llm_scheme: String,
    #[arg(long, default_value_t = String::from("localhost"))]
    pub llm_host: String,
    #[arg(long, default_value_t = 8080)]
    pub llm_port: u16,

    /// Model name (e.g. `INSAIT-Institute/MamayLM-Gemma-2-9B-IT-v0.1-GGUF` or `ggml-org/gemma-3-1b-it-GGUF`)
    #[arg(long)]
    pub llm_model: String,

    /// Initial message for the `content` subject (e.g. `translate to...`)
    #[arg(long)]
    pub llm_message: String,

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
    /// Loop update in seconds
    /// * None to exit on complete
    #[arg(long, short)]
    pub update: Option<u64>,
}
