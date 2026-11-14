use std::io::Read;

use crate::Secret;

pub fn client_id(arguments: &[String]) -> Result<String, String> {
    let mut id = None;
    let mut words = arguments.iter();
    while let Some(word) = words.next() {
        match word.as_str() {
            "--client-id" => {
                id = Some(
                    words
                        .next()
                        .cloned()
                        .ok_or("--client-id takes the OAuth client id")?,
                )
            }
            w if w == "--client-secret" || w.starts_with("--client-secret=") => {
                return Err("the client secret is never an argument, where every process on this machine can read it; pipe it on standard input".into());
            }
            flag if flag.starts_with('-') => {
                return Err(format!("{flag} is not a flag this walk takes"));
            }
            _ => return Err("the command line carries an argument this walk does not take".into()),
        }
    }
    id.ok_or_else(|| "--client-id is required".to_string())
}

pub fn client_secret(mut input: impl Read, is_terminal: bool) -> Result<Secret, String> {
    if is_terminal {
        return Err(
            "standard input is a terminal, where a typed secret echoes; pipe the client secret in"
                .into(),
        );
    }
    let mut held = String::new();
    input
        .read_to_string(&mut held)
        .map_err(|_| "standard input could not be read".to_string())?;
    let secret = held.trim();
    if secret.is_empty() {
        return Err("standard input carried no client secret".into());
    }
    Ok(Secret::from(secret))
}

#[cfg(test)]
mod tests;
