use crate::{config, prompt, report};
use postgres::{Client, NoTls};

pub(crate) fn run() -> Result<(), String> {
    let settings = config::load()?;
    let credentials = prompt::ask_credentials()?;

    let url = build_url(&settings, &credentials.username, &credentials.password);
    println!("\nconnection string = {url}");

    let mut client = Client::connect(&url, NoTls)
        .map_err(|error| format!("cannot connect: {}", report::describe(&error)))?;

    report::report(&mut client)
}

fn build_url(settings: &config::DbSettings, username: &str, password: &str) -> String {
    format!(
        "host={} port={} dbname={} application_name={} user={username} password={password}",
        settings.host, settings.port, settings.dbname, settings.application_name,
    )
}
