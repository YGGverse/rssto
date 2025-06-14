use chrono::{DateTime, Utc};

pub struct Time(String);

impl Time {
    pub fn init(format: String) -> Self {
        Self(format)
    }

    pub fn format(&self, value: Option<&str>) -> String {
        match value {
            Some(v) => chrono::DateTime::parse_from_rfc2822(v)
                .unwrap()
                .format(&self.0)
                .to_string(),
            None => todo!(),
        }
    }

    pub fn now(&self) -> String {
        utc().format(&self.0).to_string()
    }
}

pub fn utc() -> DateTime<Utc> {
    let s = std::time::SystemTime::now();
    let c: chrono::DateTime<chrono::Utc> = s.into();
    c
}
