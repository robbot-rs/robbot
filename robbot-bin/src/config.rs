use robbot::Error;
use robbot_core::config::Config;

use std::fs::File;
use std::io::Read;
use std::path::Path;

/// Reads the [`Config`] from the file with the given `path`.
pub fn from_file<P>(path: P) -> Result<Config, Error>
where
    P: AsRef<Path>,
{
    let mut file = File::open(path)?;

    let mut buf = Vec::new();
    file.read_to_end(&mut buf)?;

    let config = toml::from_slice(&buf)?;

    Ok(config)
}
