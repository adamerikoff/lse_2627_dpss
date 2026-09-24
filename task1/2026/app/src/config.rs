use serde::Deserialize;
use std::{env, fs};

const FILE_NAME: &str = "config.toml";

#[derive(Debug, Deserialize)]
pub(crate) struct DbSettings {
    pub(crate) host: String,
    pub(crate) port: u16,
    pub(crate) dbname: String,
    pub(crate) application_name: String,
}

pub(crate) fn load() -> Result<DbSettings, String> {
    let exe =
        env::current_exe().map_err(|error| format!("cannot locate the executable: {error}"))?;
    let path = exe.with_file_name(FILE_NAME);

    let text =
        fs::read_to_string(&path).map_err(|error| format!("cannot read {path:?}: {error}"))?;
    let settings =
        toml::from_str(&text).map_err(|error| format!("cannot parse {path:?}: {error}"))?;

    Ok(settings)
}
