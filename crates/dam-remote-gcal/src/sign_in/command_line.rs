//! What the sign-in binary reads before it asks Google for anything: the
//! client id from its arguments, and the client secret from a pipe.

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
mod tests {
    use super::*;

    fn words(line: &str) -> Vec<String> {
        line.split_whitespace().map(str::to_string).collect()
    }

    #[test]
    fn the_client_id_is_the_one_flag() {
        assert_eq!(
            client_id(&words("--client-id 123.apps")),
            Ok("123.apps".to_string())
        );
        assert!(client_id(&words("")).unwrap_err().contains("--client-id"));
        assert!(
            client_id(&words("--client-id"))
                .unwrap_err()
                .contains("--client-id")
        );
        assert!(
            client_id(&words("--client-id a --verbose"))
                .unwrap_err()
                .contains("--verbose")
        );
    }

    /// A stray word may be the secret itself, pasted in the wrong place, so a
    /// word that is not a flag is refused without being repeated.
    #[test]
    fn a_stray_argument_is_refused_without_being_quoted() {
        let said = client_id(&words("--client-id a GOCSPX-stray")).unwrap_err();
        assert!(!said.contains("GOCSPX-stray"), "{said}");
        assert!(
            said.contains("an argument this walk does not take"),
            "{said}"
        );
    }

    /// A secret in argv is readable by every process on the machine, so the
    /// flag is refused by name rather than failing as a typo would.
    #[test]
    fn a_client_secret_on_the_command_line_is_refused_by_name() {
        for line in [
            "--client-id a --client-secret s",
            "--client-id a --client-secret=s",
        ] {
            let said = client_id(&words(line)).unwrap_err();
            assert!(said.contains("standard input"), "{said}");
            assert!(!said.contains("=s"), "{said}");
        }
    }

    /// Typed at a terminal the secret echoes into scrollback, so a terminal
    /// on standard input is refused and the message says how to pipe it.
    #[test]
    fn the_secret_is_read_from_a_pipe_and_refused_from_a_terminal() {
        assert_eq!(
            client_secret("GOCSPX-abc\n".as_bytes(), false)
                .unwrap()
                .expose(),
            "GOCSPX-abc"
        );
        assert!(
            client_secret("GOCSPX-abc\n".as_bytes(), true)
                .unwrap_err()
                .contains("pipe")
        );
        assert!(
            client_secret(" \n".as_bytes(), false)
                .unwrap_err()
                .contains("no client secret")
        );
    }
}
