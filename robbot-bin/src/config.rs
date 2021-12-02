use robbot_core::config::Config;
use std::{fs::File, io::Read, path::Path};

pub fn from_file<P>(path: P) -> Config
where
    P: AsRef<Path>,
{
    let mut file = File::open(path).unwrap();
    let mut buf = Vec::new();
    file.read_to_end(&mut buf).unwrap();

    toml::from_slice(&buf).unwrap()
}
