use anyhow::{Result, bail};

#[derive(PartialEq)]
pub enum Level {
    //Debug,
    //Error,
    Info,
}

impl Level {
    fn parse(value: char) -> Result<Self> {
        match value {
            //'d' => Ok(Self::Debug),
            //'e' => Ok(Self::Error),
            'i' => Ok(Self::Info),
            _ => bail!("Unsupported debug value `{value}`!"),
        }
    }
}

pub struct Debug(Vec<Level>);

impl Debug {
    pub fn init(values: &str) -> Result<Self> {
        let mut l = Vec::with_capacity(values.len());
        for s in values.to_lowercase().chars() {
            l.push(Level::parse(s)?);
        }
        Ok(Self(l))
    }

    /* @TODO
    pub fn error(&self, message: &str) {
        if self.has(Level::Error) {
            eprintln!("[{}] [error] {message}", t());
        }
    } */

    pub fn info(&self, message: &str) {
        if self.0.contains(&Level::Info) {
            println!("[{}] [info] {message}", t());
        }
    }
}

fn t() -> String {
    crate::time::utc().to_rfc3339()
}
