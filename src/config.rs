use std::path::PathBuf;
use serde::{Deserialize, Serialize};
use crate::Result;
use tokio::io::AsyncReadExt;

pub async fn get_config_from_path(path: PathBuf) -> Result<LaatConfig> {
    let mut file = tokio::fs::File::open(path).await?;
    let mut contents = String::new();
    file.read_to_string(&mut contents).await?;
    let config: LaatConfig = toml::from_str(&contents)?;

    debug!("Extra: {:?}", config.extra);

    Ok(config)
}

#[derive(Deserialize, Serialize, Clone, Debug)]
pub struct LaatConfig {
    pub prefix: String,
    pub name: String,

    #[serde(default = "default_build_path")]
    pub build_path: String,
    #[serde(default = "default_assets_path")]
    pub assets_path: String,
    #[serde(default = "default_addons_path")]
    pub addons_path: String,
    #[serde(default = "default_release_path")]
    pub release_path: String,

    #[serde(default)]
    pub plugins: Vec<String>,

    #[serde(default)]
    pub pack: PackConfig,

    #[serde(flatten)]
    pub extra: toml::Value
}

#[derive(Deserialize, Serialize, Clone, Debug, Default)]
pub struct PackConfig {
    #[serde(default)]
    pub include_folders: Vec<PathBuf>,
    #[serde(default)]
    pub excludes: Vec<String>,
    #[serde(default)]
    pub header_extensions: Vec<String>
}

fn default_build_path() -> String {
    "build".to_string()
}

fn default_assets_path() -> String {
    "assets".to_string()
}

fn default_addons_path() -> String {
    "addons".to_string()
}

fn default_release_path() -> String {
    "release".to_string()
}
