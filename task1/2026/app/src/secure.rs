use crate::{config, prompt, report};
use postgres::NoTls;

pub(crate) fn run() -> Result<(), String> {
    let settings = config::load()?;
    let credentials = prompt::ask_credentials()?;

    let mut connection = postgres::Config::new();
    connection
        .host(&settings.host)
        .port(settings.port)
        .dbname(&settings.dbname)
        .application_name(&settings.application_name)
        .user(&credentials.username)
        .password(&credentials.password);

    let mut client = connection
        .connect(NoTls)
        .map_err(|error| format!("cannot connect: {}", report::describe(&error)))?;

    report::report(&mut client)
}
