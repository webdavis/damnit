mod loopback;
mod support;

use std::collections::HashMap;
use std::io::{BufRead, BufReader, Read, Write};
use std::net::TcpStream;
use std::process::{Command, Stdio};

use loopback::Reply;

const CLIENT_SECRET: &str = "GOCSPX-SUPERSECRETCLIENT";
const REFRESH: &str = "1//0gSUPERSECRETREFRESH";

fn sign_in(base: &str, home: &std::path::Path) -> Command {
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

#[test]
fn the_refresh_token_alone_reaches_standard_output() {
    let _guard = support::guard("the_refresh_token_alone_reaches_standard_output");
    let home = tempfile::tempdir().unwrap();
    let mut routes = HashMap::new();
    routes.insert(
        "POST /token",
        vec![Reply::json(
            200,
            serde_json::json!({"refresh_token": REFRESH, "access_token": "ya29.A"}),
        )],
    );
    let google = loopback::serve(routes);
    let mut child = sign_in(&google.base, home.path()).spawn().unwrap();
    child
        .stdin
        .take()
        .unwrap()
        .write_all(format!("{CLIENT_SECRET}\n").as_bytes())
        .unwrap();
    let mut stderr = BufReader::new(child.stderr.take().unwrap());
    let url = loop {
        let mut line = String::new();
        assert!(
            stderr.read_line(&mut line).unwrap() > 0,
            "the URL never arrived"
        );
        if let Some(url) = line
            .trim()
            .strip_prefix(&format!("{}/o/oauth2/v2/auth?", google.base))
        {
            break url.to_string();
        }
    };
    let param = |name: &str| {
        url.split('&')
            .find_map(|p| p.strip_prefix(&format!("{name}=")))
            .unwrap()
            .replace("%3A", ":")
            .replace("%2F", "/")
    };
    let address = param("redirect_uri")
        .trim_start_matches("http://")
        .to_string();
    let mut browser = TcpStream::connect(&address).unwrap();
    browser
        .write_all(
            format!(
                "GET /?state={}&code=4%2F0AbCODE HTTP/1.1\r\n\r\n",
                param("state")
            )
            .as_bytes(),
        )
        .unwrap();
    let mut rest = String::new();
    stderr.read_to_string(&mut rest).unwrap();
    let out = child.wait_with_output().unwrap();
    assert!(out.status.success(), "{rest}");
    assert_eq!(
        String::from_utf8(out.stdout).unwrap(),
        format!("{REFRESH}\n")
    );
    assert!(rest.contains("vault"), "{rest}");
    for secret in [CLIENT_SECRET, REFRESH, "4/0AbCODE"] {
        assert!(!rest.contains(secret), "standard error carried {secret}");
    }
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
