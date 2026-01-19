use serde::Deserialize;
use std::path::PathBuf;

#[derive(Debug, Deserialize, Default)]
pub struct Config {
    #[serde(default, rename = "file")]
    pub files: Vec<FileConfig>,
}

#[derive(Debug, Deserialize)]
pub struct FileConfig {
    pub src: PathBuf,
}
