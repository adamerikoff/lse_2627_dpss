mod config;
mod insecure;
mod mode;
mod prompt;
mod report;
mod secure;

use crate::mode::Mode;
use std::process::ExitCode;

// One iteration's outcome: keep looping, or the user asked to quit (EOF).
enum Flow {
    Continue,
    Quit,
}

fn main() -> ExitCode {
    // Config is read once: host/port/dbname/application_name do not change
    // between attempts. Only the mode and credentials are asked each loop.
    let settings = match config::load() {
        Ok(settings) => settings,
        Err(message) => {
            eprintln!("error: {message}");
            return ExitCode::FAILURE;
        }
    };

    loop {
        match session(&settings) {
            Ok(Flow::Continue) => println!(),
            Ok(Flow::Quit) => {
                println!("bye");
                return ExitCode::SUCCESS;
            }
            // A failed attempt (e.g. a rejected injection payload, or a wrong
            // password) is expected in this demo. Report it and keep looping so
            // the next payload can be tried without restarting the process.
            Err(message) => eprintln!("error: {message}"),
        }
    }
}

fn session(settings: &config::DbSettings) -> Result<Flow, String> {
    let mode = match prompt::ask_mode()? {
        Some(mode) => mode,
        None => return Ok(Flow::Quit),
    };

    let credentials = match prompt::ask_credentials()? {
        Some(credentials) => credentials,
        None => return Ok(Flow::Quit),
    };

    match mode {
        Mode::Secure => secure::run(settings, &credentials)?,
        Mode::Insecure => insecure::run(settings, &credentials)?,
    }

    Ok(Flow::Continue)
}
