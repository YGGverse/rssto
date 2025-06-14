use anyhow::{Result, bail};
use std::{fs, path::PathBuf, str::FromStr};

pub struct Target(Vec<PathBuf>);

impl Target {
    pub fn init(paths: &Vec<String>) -> Result<Self> {
        let mut t = Vec::with_capacity(paths.len());
        for path in paths {
            let p = PathBuf::from_str(path)?;
            if fs::metadata(&p).is_ok_and(|t| t.is_file()) {
                bail!("Target destination exists and not directory!")
            }
            fs::create_dir_all(&p)?;
            t.push(p)
        }
        Ok(Self(t))
    }

    pub fn index(&self, index: usize, extension: &str) -> PathBuf {
        let mut p = PathBuf::from(&self.0[index]);
        p.push(format!("index.{extension}"));
        p
    }
}
