mod argument;

use anyhow::Result;

#[tokio::main]
async fn main() -> Result<()> {
    use argument::Argument;
    use chrono::Local;
    use clap::Parser;
    use lancor::{ChatCompletionRequest, LlamaCppClient, Message};
    use log::{debug, info};
    use mysql::{Mysql, Sort};
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

    let arg = Argument::parse();
    let db = Mysql::connect(
        &arg.mysql_host,
        arg.mysql_port,
        &arg.mysql_username,
        &arg.mysql_password,
        &arg.mysql_database,
    )?;
    let llm = LlamaCppClient::new(format!(
        "{}://{}:{}",
        arg.llm_scheme, arg.llm_host, arg.llm_port
    ))?;

    let provider_id = match db.provider_by_name(&arg.llm_model)? {
        Some(p) => {
            debug!(
                "Use existing DB provider #{} matches model name `{}`",
                p.provider_id, &arg.llm_model
            );
            p.provider_id
        }
        None => {
            let provider_id = db.insert_provider(&arg.llm_model)?;
            info!(
                "Provider `{}` not found in database, created new one with ID `{provider_id}`",
                &arg.llm_model
            );
            provider_id
        }
    };

    info!("Daemon started");
    loop {
        debug!("New queue begin...");
        for source in db.contents_queue_for_provider_id(provider_id, Sort::Asc, None)? {
            debug!(
                "Begin generating `content_id` #{} using `provider_id` #{provider_id}.",
                source.content_id
            );

            let title =
                llm.chat_completion(ChatCompletionRequest::new(&arg.llm_model).message(
                    Message::user(format!("{}\n{}", arg.llm_message, source.title)),
                ))
                .await?;

            println!("{}", &title.choices[0].message.content);

            let description =
                llm.chat_completion(ChatCompletionRequest::new(&arg.llm_model).message(
                    Message::user(format!("{}\n{}", arg.llm_message, source.description)),
                ))
                .await?;

            let content_id = db.insert_content(
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
        debug!("Queue completed");
        if let Some(update) = arg.update {
            debug!("Wait {update} seconds to continue...");
            std::thread::sleep(std::time::Duration::from_secs(update))
        } else {
            return Ok(());
        }
    }
}
