mod argument;
mod config;

use anyhow::Result;
use mysql::Database;

#[tokio::main]
async fn main() -> Result<()> {
    use chrono::Local;
    use clap::Parser;
    use lancor::{ChatCompletionRequest, LlamaCppClient, Message};
    use log::{debug, info};

    use std::env::var;

    if var("RUST_LOG").is_ok() {
        use tracing_subscriber::{EnvFilter, fmt::*};
        struct T;
        impl time::FormatTime for T {
            fn format_time(&self, w: &mut format::Writer<'_>) -> std::fmt::Result {
                write!(w, "{}", Local::now())
            }
        }
        fmt()
            .with_timer(T)
            .with_env_filter(EnvFilter::from_default_env())
            .init()
    }

    let argument = argument::Argument::parse();
    let config: config::Config = toml::from_str(&std::fs::read_to_string(argument.config)?)?;

    let llm = LlamaCppClient::new(format!(
        "{}://{}:{}",
        config.llm.scheme, config.llm.host, config.llm.port
    ))?;
    let db = Database::pool(
        &config.mysql.host.to_string(),
        config.mysql.port,
        &config.mysql.username,
        &config.mysql.password,
        &config.mysql.database,
    )?;

    let provider_id = {
        let mut conn = db.connection()?;
        match conn.provider_id_by_name(&config.llm.model)? {
            Some(provider_id) => {
                debug!(
                    "Use existing DB provider #{} matches model name `{}`",
                    provider_id, &config.llm.model
                );
                provider_id
            }
            None => {
                let provider_id = conn.insert_provider(&config.llm.model)?;
                info!(
                    "Provider `{}` not found in database, created new one with ID `{provider_id}`",
                    &config.llm.model
                );
                provider_id
            }
        }
    };

    info!("Daemon started");
    loop {
        debug!("New queue begin...");
        let mut tx = db.transaction()?;
        for source in tx.contents_queue_for_provider_id(provider_id)? {
            debug!(
                "Begin generating `content_id` #{} using `provider_id` #{provider_id}.",
                source.content_id
            );

            let title = llm
                .chat_completion(ChatCompletionRequest::new(&config.llm.model).message(
                    Message::user(format!("{}\n{}", config.llm.message, source.title)),
                ))
                .await?;

            let description = llm
                .chat_completion(ChatCompletionRequest::new(&config.llm.model).message(
                    Message::user(format!("{}\n{}", config.llm.message, source.description)),
                ))
                .await?;

            let content_id = tx.insert_content(
                source.channel_item_id,
                Some(provider_id),
                &title.choices[0].message.content,
                &description.choices[0].message.content,
            )?;

            debug!(
                "Created `content_id` #{content_id} using `content_id` #{} source by `provider_id` #{provider_id}.",
                source.content_id
            )
        }
        tx.commit()?;
        debug!("Queue completed");
        if let Some(update) = config.update {
            debug!("Wait {update} seconds to continue...");
            std::thread::sleep(std::time::Duration::from_secs(update))
        } else {
            return Ok(());
        }
    }
}
