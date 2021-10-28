use serde::Deserialize;
use std::{fs::File, io::Read, path::Path};

#[derive(Deserialize)]
pub struct Config {
    pub token: String,
    pub database: Database,
}

impl Config {
    pub fn load<P>(path: P) -> Self
    where
        P: AsRef<Path>,
    {
        let mut file = File::open(path).unwrap();
        let mut buf = Vec::new();
        file.read_to_end(&mut buf).unwrap();

        toml::from_slice(&buf).unwrap()
    }
}

/// Database configuration section. Note that not all
/// fields are required for all driver types.
#[derive(Deserialize)]
pub struct Database {
    pub driver: String,
    pub host: String,
    pub port: u16,
    pub user: String,
    pub password: String,
    pub database: String,
}
