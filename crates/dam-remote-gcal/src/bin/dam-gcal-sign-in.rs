//! The operator's one-time walk: consent in the browser, the refresh token on
//! standard output, and nothing written anywhere.

use std::io::{IsTerminal, Write};

use dam_remote_gcal::sign_in::{client_id, client_secret};
use dam_remote_gcal::{Client, Endpoints, SignIn};

const USAGE: &str = "usage: dam-gcal-sign-in --client-id <id>
  the client secret arrives on standard input, for example
  security find-generic-password -w -s 'Google dam client' | dam-gcal-sign-in --client-id <id>";

const KEEP_IT: &str = "Store the refresh token printed on standard output in your vault now: it is \
shown once and written nowhere. The gcal remote's refresh_token_command reads it from there.";

const LOST: &str = "the refresh token could not be written to standard output, and it is kept \
nowhere else; sign in again";

fn main() {
    let arguments: Vec<String> = std::env::args().skip(1).collect();
    let stdin = std::io::stdin();
    let is_terminal = stdin.is_terminal();
    let asked = client_id(&arguments).and_then(|id| {
        Ok(Client {
            id,
            secret: client_secret(stdin.lock(), is_terminal)?,
        })
    });
    let client = match asked {
        Ok(client) => client,
        Err(why) => {
            eprintln!("dam-gcal-sign-in: {why}\n{USAGE}");
            std::process::exit(2);
        }
    };
    let endpoints = match Endpoints::from_env() {
        Ok(e) => e,
        Err(why) => {
            eprintln!("dam-gcal-sign-in: {why}");
            std::process::exit(2);
        }
    };
    let minted = SignIn::new(endpoints).mint(&client, &mut |url| {
        eprintln!("Open this URL, grant access, and come back:\n\n{url}\n");
    });
    match minted {
        Ok(token) => {
            if writeln!(std::io::stdout().lock(), "{}", token.expose()).is_err() {
                eprintln!("dam-gcal-sign-in: {LOST}");
                std::process::exit(1);
            }
            eprintln!("{KEEP_IT}");
        }
        Err(why) => {
            eprintln!("dam-gcal-sign-in: {why}");
            std::process::exit(1);
        }
    }
}
