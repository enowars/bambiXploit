use std::{
    fs,
    io,
};

use serde::Deserialize;

#[derive(Debug)]
pub enum LoadConfigError {
    CouldNotReadConfig(io::Error),
    CouldNotParseConfig(serde_json::Error),
}

impl From<io::Error> for LoadConfigError {
    fn from(error: io::Error) -> Self {
        LoadConfigError::CouldNotReadConfig(error)
    }
}

impl From<serde_json::Error> for LoadConfigError {
    fn from(error: serde_json::Error) -> Self {
        LoadConfigError::CouldNotParseConfig(error)
    }
}

#[derive(Deserialize, Debug)]
pub struct Config {
    pub flag_re: String,
    pub addresses: Vec<String>,
    pub flagbot_address: String,
    pub interval: usize,
}

pub fn load_config(file_path: Option<&str>) -> Result<Config, LoadConfigError> {
    let path = file_path.unwrap_or("/bambiXploit.conf");
    Ok(serde_json::from_str(&fs::read_to_string(path)?)?)
}
