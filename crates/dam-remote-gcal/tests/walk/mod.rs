#![allow(dead_code)]

use std::io::{Read, Write};
use std::net::TcpStream;

use dam_remote_gcal::{Client, Endpoints, Secret, SignIn, SignInError};
use sha2::Digest;

use crate::loopback::{self, Loopback};

pub const CLIENT_SECRET: &str = "GOCSPX-SUPERSECRETCLIENT";
pub const REFRESH: &str = "1//0gSUPERSECRETREFRESH";

pub fn client() -> Client {
    Client {
        id: "123.apps.googleusercontent.com".into(),
        secret: CLIENT_SECRET.into(),
    }
}

pub fn sign_in_against(google: &Loopback) -> SignIn {
    SignIn::new(Endpoints::loopback(&google.base).unwrap())
}

pub fn query_of(url: &str) -> &str {
    url.split_once('?').map(|(_, q)| q).unwrap_or_default()
}

pub fn still_encoded_field<'a>(text: &'a str, name: &str) -> Option<&'a str> {
    text.split('&')
        .find_map(|pair| pair.strip_prefix(name)?.strip_prefix('='))
}

pub fn s256_computed_apart_from_the_code_under_test(verifier: &str) -> String {
    const URL_SAFE_ALPHABET: &[u8; 64] =
        b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789-_";
    let digest = sha2::Sha256::digest(verifier.as_bytes());
    let mut out = String::new();
    for chunk in digest.chunks(3) {
        let n = chunk
            .iter()
            .enumerate()
            .fold(0u32, |n, (i, b)| n | u32::from(*b) << (16 - 8 * i));
        for i in 0..=chunk.len() {
            out.push(URL_SAFE_ALPHABET[(n >> (18 - 6 * i) & 63) as usize] as char);
        }
    }
    out
}

pub fn play_the_browser_returning_its_page(
    url: &str,
    redirect_query_for_state: impl Fn(&str) -> String,
) -> std::thread::JoinHandle<String> {
    let asked = loopback::decoded_fields(query_of(url));
    let field = |name: &str| asked.get(name).cloned().unwrap_or_default();
    let address = field("redirect_uri")
        .trim_start_matches("http://")
        .to_string();
    let line = format!(
        "GET /?{} HTTP/1.1\r\nHost: {address}\r\n\r\n",
        redirect_query_for_state(&field("state"))
    );
    std::thread::spawn(move || {
        let mut stream = TcpStream::connect(address).unwrap();
        stream.write_all(line.as_bytes()).unwrap();
        let mut page = String::new();
        let _ = stream.read_to_string(&mut page);
        page
    })
}

pub fn mint_with_the_browser_granting_code_c(google: &Loopback) -> Result<Secret, SignInError> {
    let mut browser = None;
    let minted = sign_in_against(google).mint(&client(), &mut |url| {
        browser = Some(play_the_browser_returning_its_page(url, |state| {
            format!("state={state}&code=c")
        }));
    });
    let _ = browser.unwrap().join();
    minted
}
