use adbir::{helpers::errors::AdbirError, Config, HomeTemplate};
use askama::Template;
use clap::Parser;
use std::{fs::OpenOptions, io::BufWriter, path::Path};

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

fn main() -> Result<(), AdbirError> {
    let args = Args::parse();
    println!("Started with args: {:?}", args);

    println!("Reading from {}", args.config_path);
    let config = Config::from_path(&args.config_path)?;

    println!("Opening output directory file");
    let out_file = OpenOptions::new()
        .create(true)
        .write(true)
        .truncate(true)
        .open(Path::new(&args.out_dir).join("index.html"))?;

    println!("Rendering and writing template to output file");
    HomeTemplate::new(config).write_into(&mut BufWriter::new(out_file))?;

    Ok(())
}
