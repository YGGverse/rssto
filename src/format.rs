use anyhow::{bail, Result};

pub struct Template {
    pub index: String,
    pub index_item: String,
}

impl Template {
    pub fn html(template_path: &str) -> Result<Self> {
        use std::{fs::read_to_string, path::PathBuf, str::FromStr};

        let mut p = PathBuf::from_str(template_path)?;
        p.push("html");

        Ok(Self {
            index: read_to_string(&{
                let mut p = PathBuf::from(&p);
                p.push("index.html");
                p
            })?,
            index_item: read_to_string(&{
                let mut p = PathBuf::from(&p);
                p.push("index");
                p.push("item.html");
                p
            })?,
        })
    }
}

pub enum Type {
    Html(Template),
}

impl Type {
    fn parse(format: &str, template_path: &str) -> Result<Self> {
        if matches!(format.to_lowercase().as_str(), "html") {
            return Ok(Self::Html(Template::html(template_path)?));
        }
        bail!("Format `{format}` support yet not implemented!")
    }
}

pub struct Format(Vec<Type>);

impl Format {
    pub fn init(values: &Vec<String>, template: &str) -> Result<Self> {
        let mut f = Vec::with_capacity(values.len());
        for s in values {
            f.push(Type::parse(s, template)?);
        }
        Ok(Self(f))
    }
    pub fn get(&self) -> &Vec<Type> {
        &self.0
    }
}
