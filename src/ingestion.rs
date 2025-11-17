// the ingestinator

use std::{
    fs::{self, File},
    io::Read,
    path::Path,
};

// scan pre-designated folders for files
// grab files
// hash and compare to list of processed files
use serde::Deserialize;
use toml;

use crate::utils::csvutil::{ingest_amex, ingest_citizens};

#[derive(Deserialize, Debug)]
struct Config {
    folders: Folders,
}
#[derive(Deserialize, Debug)]
struct Folders {
    amex: String,
    citizens: String,
    apple: String,
    chase: String,
}

pub fn ingestinator() -> std::io::Result<()> {
    let config_path = Path::new("./config/moneyd-config.toml");
    let absolute_config = fs::canonicalize(config_path)?;
    let mut config_file = File::open(absolute_config)?;
    let mut config_contents = String::new();
    config_file.read_to_string(&mut config_contents)?;

    let config: Config = toml::from_str(&config_contents.as_str()).unwrap();

    let amex_dir = fs::read_dir(config.folders.amex).unwrap();
    let mut citizens_dir = fs::read_dir(config.folders.citizens).unwrap();
    for path in amex_dir {
        // TODO: need to do hashing and checks here
        let file_path = path
            .unwrap()
            .path()
            .canonicalize()
            .unwrap()
            .as_os_str()
            .to_str()
            .expect("Invalid file path: amex")
            .to_string();
        let r = ingest_amex(file_path).unwrap();
    }

    for path in citizens_dir {
        let file_path = path
            .unwrap()
            .path()
            .canonicalize()
            .unwrap()
            .as_os_str()
            .to_str()
            .expect("Invalid file path: citizens")
            .to_string();
        let citizens_res = ingest_citizens(file_path).unwrap();

        println!("{:?}", citizens_res);
    }

    Ok(())
}
