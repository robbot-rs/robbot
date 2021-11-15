use log::LevelFilter;
use serde::{Deserialize, Serialize};
use std::{fs::File, io::Read, path::Path};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Config {
    pub token: String,
    pub loglevel: LevelFilter,
    pub database: Database,
    pub superusers: Vec<u64>,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            token: String::new(),
            loglevel: LevelFilter::Info,
            database: Database::default(),
            superusers: Vec::new(),
        }
    }
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
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct Database {
    pub driver: String,
    pub host: String,
    pub port: u16,
    pub user: String,
    pub password: String,
    pub database: String,
}

impl Database {
    pub fn connect_string(&self) -> String {
        format!(
            "{}://{}:{}@{}:{}/{}?ssl-mode=DISABLED",
            self.driver, self.user, self.password, self.host, self.port, self.database
        )
    }
}

#[cfg(test)]
mod tests {
    use super::Database;

    #[test]
    fn test_database_connect_string() {
        let database = Database {
            driver: String::from("mysql"),
            host: String::from("127.0.0.1"),
            port: 3306,
            user: String::from("robbot"),
            password: String::from("pw"),
            database: String::from("db"),
        };

        assert_eq!(
            database.connect_string(),
            "mysql://robbot:pw@127.0.0.1:3306/db?ssl-mode=DISABLED"
        )
    }
}
