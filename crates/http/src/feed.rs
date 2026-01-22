/// Export crawl index to the RSS file
pub struct Feed {
    buffer: String,
}

impl Feed {
    pub fn new(title: &str, description: Option<&str>, capacity: usize) -> Self {
        let t = chrono::Utc::now().to_rfc2822();
        let mut buffer = String::with_capacity(capacity);

        buffer.push_str("<?xml version=\"1.0\" encoding=\"UTF-8\"?><rss version=\"2.0\"><channel>");

        buffer.push_str(&format!("<pubDate>{t}</pubDate>"));
        buffer.push_str(&format!("<lastBuildDate>{t}</lastBuildDate>"));
        buffer.push_str(&format!("<title>{}</title>", escape(title)));

        if let Some(d) = description {
            buffer.push_str(&format!("<description>{}</description>", escape(d)));
        }

        Self { buffer }
    }

    /// Append `item` to the feed `channel`
    pub fn push(
        &mut self,
        guid: u64,
        time: chrono::DateTime<chrono::Utc>,
        url: String,
        title: String,
        description: String,
    ) {
        self.buffer.push_str(&format!(
            "<item><guid>{guid}</guid><title>{}</title><link>{url}</link><description>{}</description><pubDate>{}</pubDate></item>",
            escape(&title),
            escape(&description),
            time.to_rfc2822()
        ))
    }

    /// Write final bytes
    pub fn commit(mut self) -> String {
        self.buffer.push_str("</channel></rss>");
        self.buffer
    }
}

// @TODO use tera filters?
// https://keats.github.io/tera/docs/#built-in-filters

fn escape(value: &str) -> String {
    value
        .replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace("'", "&apos;")
}
