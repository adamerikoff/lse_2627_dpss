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
    let mut settings: DbSettings =
        toml::from_str(&text).map_err(|error| format!("cannot parse {path:?}: {error}"))?;

    // Env overrides let ONE binary run in two places without editing the file:
    //   - on the host: config.toml's 127.0.0.1:55432 (the published port)
    //   - in compose:  APP_DB_HOST=db, APP_DB_PORT=5432 (the internal service)
    if let Ok(host) = env::var("APP_DB_HOST") {
        settings.host = host;
    }
    if let Ok(port) = env::var("APP_DB_PORT") {
        settings.port = port
            .parse()
            .map_err(|error| format!("invalid APP_DB_PORT {port:?}: {error}"))?;
    }

    Ok(settings)
}
