pub mod helpers;

use askama::Template;
use serde::Deserialize;
use std::{fs::File, io::BufReader, path::Path};

use crate::helpers::errors::AdbirError;

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Service {
    name: String,
    url: String,
    logo: Option<String>,
    subtitle: String,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ServiceGroup {
    name: String,
    items: Vec<Service>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Config {
    title: String,
    subtitle: Option<String>,
    image: Option<String>,
    services: Vec<ServiceGroup>,
}

impl Config {
    pub fn from_path(config_path: impl AsRef<Path>) -> Result<Self, AdbirError> {
        Ok(serde_yaml::from_reader(BufReader::new(File::open(
            config_path,
        )?))?)
    }
}

#[derive(Debug, Template)]
#[template(path = "index.html")]
pub struct HomeTemplate {
    pub config: Config,
}

impl HomeTemplate {
    pub fn new(config: Config) -> Self {
        Self { config }
    }
}
