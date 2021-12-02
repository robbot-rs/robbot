use log::LevelFilter;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Config {
    pub token: String,
    pub loglevel: LevelFilter,
    pub database: Database,
    pub admins: Vec<u64>,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            token: String::new(),
            loglevel: LevelFilter::Info,
            database: Database::default(),
            admins: Vec::new(),
        }
    }
}

/// Database configuration section. Not all
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
