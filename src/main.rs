mod helpers;

use askama::Template;
use clap::Parser;
use serde::Deserialize;
use std::{
    fs::{File, OpenOptions},
    io::{BufReader, BufWriter},
    path::Path,
};

use crate::helpers::errors::AbdirError;

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct Service {
    name: String,
    url: String,
    logo: String,
    subtitle: String,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct ServiceGroup {
    name: String,
    items: Vec<Service>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct Config {
    title: String,
    subtitle: String,
    services: Vec<ServiceGroup>,
}

#[derive(Debug, Parser)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// directory to output generated resources
    #[arg(long, env, default_value = "./out")]
    out_dir: String,

    /// path to config file
    #[arg(long, env, default_value = "./config.yaml")]
    config_path: String,
}

#[derive(Debug, Template)]
#[template(path = "index.html")]
struct HomeTemplate {
    config: Config,
}

fn main() -> Result<(), AbdirError> {
    let args = Args::parse();
    println!("Started with args: {:?}", args);

    println!("Reading from {}", args.config_path);
    let config: Config = serde_yaml::from_reader(BufReader::new(File::open(&args.config_path)?))?;

    println!("Opening output directory file");
    let out_file = OpenOptions::new()
        .create(true)
        .write(true)
        .truncate(true)
        .open(Path::new(&args.out_dir).join("index.html"))?;

    println!("Rendering and writing template to output file");
    HomeTemplate { config }.write_into(&mut BufWriter::new(out_file))?;

    Ok(())
}
