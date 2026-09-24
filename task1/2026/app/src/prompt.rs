use crate::mode::Mode;
use std::io::{self, Write};

pub(crate) struct Credentials {
    pub(crate) username: String,
    pub(crate) password: String,
}

pub(crate) fn ask_mode() -> Result<Mode, String> {
    loop {
        let answer = ask("mode [s]ecure / [i]nsecure: ")?;

        match Mode::parse(&answer) {
            Some(mode) => return Ok(mode),
            None => eprintln!("expected `s` or `i`, got {answer:?}"),
        }
    }
}

pub(crate) fn ask_credentials() -> Result<Credentials, String> {
    let username = ask("username: ")?;
    let password = rpassword::prompt_password("password: ")
        .map_err(|error| format!("cannot read password: {error}"))?;

    Ok(Credentials { username, password })
}

fn ask(question: &str) -> Result<String, String> {
    print!("{question}");
    io::stdout()
        .flush()
        .map_err(|error| format!("cannot write to stdout: {error}"))?;

    let mut line = String::new();
    let bytes = io::stdin()
        .read_line(&mut line)
        .map_err(|error| format!("cannot read from stdin: {error}"))?;

    if bytes == 0 {
        return Err("end of input".to_owned());
    }

    Ok(line.trim().to_owned())
}
