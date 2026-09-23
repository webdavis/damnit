mod loopback;
mod support;

use std::collections::HashMap;
use std::io::{BufRead, BufReader, Read, Write};
use std::net::TcpStream;
use std::path::Path;
use std::process::{Command, Stdio};

use loopback::Reply;

const CLIENT_SECRET: &str = "GOCSPX-SUPERSECRETCLIENT";
const REFRESH: &str = "1//0gSUPERSECRETREFRESH";

fn sign_in(base: &str, home: &Path) -> Command {
    let mut command = Command::new(env!("CARGO_BIN_EXE_dam-gcal-sign-in"));
    command
        .args(["--client-id", "123.apps.googleusercontent.com"])
        .env("DAM_GCAL_BASE_URL", base)
        .env("HOME", home)
        .env("XDG_CONFIG_HOME", home.join("config"))
        .env("XDG_DATA_HOME", home.join("data"))
        .env("XDG_STATE_HOME", home.join("state"))
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());
    command
}

/// What one run of the binary left behind.
struct Walked {
    code: Option<i32>,
    stdout: String,
    stderr: String,
}

/// One run: the secret piped in, the browser played with `answer` given the
/// announced state, and every byte of both streams kept. With `keep_stdout`
/// false the read end of standard output is closed before the token is due.
fn walk(base: &str, home: &Path, answer: impl Fn(&str) -> String, keep_stdout: bool) -> Walked {
    let mut child = sign_in(base, home).spawn().unwrap();
    child
        .stdin
        .take()
        .unwrap()
        .write_all(format!("{CLIENT_SECRET}\n").as_bytes())
        .unwrap();
    if !keep_stdout {
        drop(child.stdout.take());
    }
    let mut stderr = BufReader::new(child.stderr.take().unwrap());
    let mut said = String::new();
    let url = loop {
        let mut line = String::new();
        assert!(
            stderr.read_line(&mut line).unwrap() > 0,
            "the URL never arrived: {said}"
        );
        said.push_str(&line);
        if let Some(url) = line
            .trim()
            .strip_prefix(&format!("{base}/o/oauth2/v2/auth?"))
        {
            break url.to_string();
        }
    };
    let asked = loopback::fields(&url);
    let field = |name: &str| asked.get(name).cloned().unwrap_or_default();
    let address = field("redirect_uri")
        .trim_start_matches("http://")
        .to_string();
    let mut browser = TcpStream::connect(&address).unwrap();
    let request = format!("GET /?{} HTTP/1.1\r\n\r\n", answer(&field("state")));
    browser.write_all(request.as_bytes()).unwrap();
    stderr.read_to_string(&mut said).unwrap();
    let out = child.wait_with_output().unwrap();
    Walked {
        code: out.status.code(),
        stdout: String::from_utf8_lossy(&out.stdout).into_owned(),
        stderr: said,
    }
}

fn granted(state: &str) -> String {
    format!("state={state}&code=4%2F0AbCODE")
}

fn google_granting() -> loopback::Loopback {
    let mut routes = HashMap::new();
    routes.insert(
        "POST /token",
        vec![Reply::json(
            200,
            serde_json::json!({"refresh_token": REFRESH, "access_token": "ya29.A"}),
        )],
    );
    loopback::serve(routes)
}

fn assert_quotes_no_secret(stderr: &str) {
    for secret in [CLIENT_SECRET, REFRESH, "4/0AbCODE"] {
        assert!(!stderr.contains(secret), "standard error carried {secret}");
    }
}

#[test]
fn the_refresh_token_alone_reaches_standard_output() {
    let _guard = support::guard("the_refresh_token_alone_reaches_standard_output");
    let home = tempfile::tempdir().unwrap();
    let google = google_granting();
    let walked = walk(&google.base, home.path(), granted, true);
    assert_eq!(walked.code, Some(0), "{}", walked.stderr);
    assert_eq!(walked.stdout, format!("{REFRESH}\n"));
    assert!(walked.stderr.contains("vault"), "{}", walked.stderr);
    assert_quotes_no_secret(&walked.stderr);
    let left: Vec<_> = std::fs::read_dir(home.path())
        .unwrap()
        .flatten()
        .map(|e| e.path())
        .collect();
    assert!(left.is_empty(), "the walk wrote {left:?}");
}

#[test]
fn a_consent_refused_in_the_browser_exits_one_and_prints_nothing() {
    let _guard = support::guard("a_consent_refused_in_the_browser_exits_one_and_prints_nothing");
    let home = tempfile::tempdir().unwrap();
    let google = google_granting();
    let walked = walk(
        &google.base,
        home.path(),
        |state| format!("error=access_denied&state={state}"),
        true,
    );
    assert_eq!(walked.code, Some(1), "{}", walked.stderr);
    assert_eq!(walked.stdout, "");
    assert!(walked.stderr.contains("access_denied"), "{}", walked.stderr);
    assert_quotes_no_secret(&walked.stderr);
    assert!(google.seen().is_empty());
}

/// The token exists only in this process, so failing to print it loses it,
/// and the operator is told to walk again.
#[test]
fn a_token_that_cannot_be_printed_says_to_sign_in_again() {
    let _guard = support::guard("a_token_that_cannot_be_printed_says_to_sign_in_again");
    let home = tempfile::tempdir().unwrap();
    let google = google_granting();
    let walked = walk(&google.base, home.path(), granted, false);
    assert_eq!(walked.code, Some(1), "{}", walked.stderr);
    assert!(walked.stderr.contains("sign in again"), "{}", walked.stderr);
    assert_quotes_no_secret(&walked.stderr);
}

#[test]
fn an_unknown_flag_is_a_usage_error_before_anything_is_asked() {
    let _guard = support::guard("an_unknown_flag_is_a_usage_error_before_anything_is_asked");
    let home = tempfile::tempdir().unwrap();
    let google = loopback::serve(HashMap::new());
    let out = sign_in(&google.base, home.path())
        .arg("--open")
        .stdin(Stdio::null())
        .output()
        .unwrap();
    assert_eq!(out.status.code(), Some(2));
    assert!(String::from_utf8_lossy(&out.stderr).contains("usage"));
    assert!(google.seen().is_empty());
}
