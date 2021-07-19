use std::path::PathBuf;
use serde::Deserialize;
use std::error::Error;
use std::io::Read;

pub fn get_config_from_path(path: PathBuf) -> Result<LaatConfig, Box<dyn Error>> {
    let mut file = std::fs::File::open(path)?;
    let mut contents = String::new();
    file.read_to_string(&mut contents)?;
    let config = toml::from_str(&contents)?;

    Ok(config)
}

#[derive(Deserialize, Clone)]
pub struct LaatConfig {
    pub prefix: String,
    pub name: String,

    #[serde(default = "default_build_path")]
    pub build_path: String,
    #[serde(default = "default_assets_path")]
    pub assets_path: String,
    #[serde(default = "default_addons_path")]
    pub addons_path: String,
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
