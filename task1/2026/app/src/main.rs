mod config;
mod insecure;
mod mode;
mod prompt;
mod report;
mod secure;

use crate::mode::Mode;
use std::process::ExitCode;

fn main() -> ExitCode {
    match run() {
        Ok(()) => ExitCode::SUCCESS,
        Err(message) => {
            eprintln!("error: {message}");
            ExitCode::FAILURE
        }
    }
}

fn run() -> Result<(), String> {
    let mode = prompt::ask_mode()?;

    match mode {
        Mode::Secure => secure::run(),
        Mode::Insecure => insecure::run(),
    }
}
