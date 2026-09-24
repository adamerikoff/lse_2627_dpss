use postgres::Client;
use std::error::Error;

pub(crate) fn describe(error: &postgres::Error) -> String {
    let mut text = error.to_string();
    let mut source = error.source();

    while let Some(cause) = source {
        text.push_str(": ");
        text.push_str(&cause.to_string());
        source = cause.source();
    }

    text
}

pub(crate) fn report(client: &mut Client) -> Result<(), String> {
    let row = client
        .query_one(
            "SELECT version(), current_database(), current_user, session_user, current_setting('application_name'), current_setting('search_path')",
            &[],
        )
        .map_err(|error| format!("query failed: {}", describe(&error)))?;

    let version: String = row.get(0);
    let database: String = row.get(1);
    let user: String = row.get(2);
    let session_user: String = row.get(3);
    let application_name: String = row.get(4);
    let search_path: String = row.get(5);

    println!();
    println!("==================================================");
    println!("               POSTGRESQL SESSION REPORT          ");
    println!("==================================================");
    println!("version          = {version}");
    println!("database         = {database}");
    println!("user             = {user}");
    println!("session_user     = {session_user}");
    println!("application_name = {application_name}");
    println!("search_path      = {search_path}");
    println!("==================================================");

    // UNQUALIFIED on purpose: `data` resolves against whatever database this
    // connection landed on. In `publicdb` it is public.data (innocuous rows); if
    // the insecure builder let the user inject `dbname=secretdb`, the identical
    // query now reads secretdb's data table and leaks its rows. Same SQL — the
    // connection string alone decided the target.
    let rows = client
        .query("SELECT id, label FROM data ORDER BY id", &[])
        .map_err(|error| format!("data query failed: {}", describe(&error)))?;

    println!();
    println!("--------------------------------------------------");
    println!("               TABLE data ({} rows)", rows.len());
    println!("--------------------------------------------------");
    println!("  {:>3}  |  {}", "id", "label");
    println!("  -----+-------------------------------------------");
    for row in &rows {
        let id: i32 = row.get(0);
        let label: String = row.get(1);
        println!("  {id:>3}  |  {label}");
    }
    println!("--------------------------------------------------");

    Ok(())
}