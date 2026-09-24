use crate::mode::Mode;
use std::io::{self, Write};

pub(crate) struct Credentials {
    pub(crate) username: String,
    pub(crate) password: String,
}

/// Asks for the mode, reprompting on invalid input.
/// Returns `Ok(None)` on end-of-input (Ctrl-D), which the caller treats as quit.
pub(crate) fn ask_mode() -> Result<Option<Mode>, String> {
    loop {
        let answer = match ask("mode [s]ecure / [i]nsecure: ")? {
            Some(answer) => answer,
            None => return Ok(None),
        };

        match Mode::parse(&answer) {
            Some(mode) => return Ok(Some(mode)),
            None => eprintln!("expected `s` or `i`, got {answer:?}"),
        }
    }
}

/// Asks for username (stdin) and password (read from /dev/tty by rpassword).
/// Returns `Ok(None)` if the username prompt hits end-of-input (quit).
pub(crate) fn ask_credentials() -> Result<Option<Credentials>, String> {
    let username = match ask("username: ")? {
        Some(username) => username,
        None => return Ok(None),
    };

    let password = rpassword::prompt_password("password: ")
        .map_err(|error| format!("cannot read password: {error}"))?;

    Ok(Some(Credentials { username, password }))
}

/// Reads one trimmed line from stdin. `Ok(None)` signals EOF (0 bytes read).
fn ask(question: &str) -> Result<Option<String>, String> {
    print!("{question}");
    io::stdout()
        .flush()
        .map_err(|error| format!("cannot write to stdout: {error}"))?;

    let mut line = String::new();
    let bytes = io::stdin()
        .read_line(&mut line)
        .map_err(|error| format!("cannot read from stdin: {error}"))?;

    if bytes == 0 {
        return Ok(None);
    }

    Ok(Some(line.trim().to_owned()))
}
