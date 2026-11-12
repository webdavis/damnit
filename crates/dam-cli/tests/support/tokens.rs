use dam_domain::Token;

use super::errors::error_document;
use super::sandbox::Sandbox;
use super::schema::assert_matches_schema;

pub fn token(fill: u8) -> Token {
    Token::generate(&mut |b: &mut [u8]| b.fill(fill))
}

pub fn grant_to(sb: &Sandbox, name: &str, fill: u8, grants: &str) -> String {
    let held = token(fill).expose();
    sb.add_config(&format!("[token.{name}]\nvalue = \"{held}\"\n{grants}"));
    held
}

pub fn grant(allow: &str) -> String {
    format!("[[token.vpt.grant]]\nallow = [{allow}]\n")
}

pub fn answer(out: &std::process::Output, schema_file: &str) -> serde_json::Value {
    assert!(out.status.success(), "{out:?}");
    let document: serde_json::Value = serde_json::from_slice(&out.stdout).unwrap();
    assert_matches_schema(&document, schema_file);
    document
}

pub fn refused(out: &std::process::Output) -> String {
    assert_eq!(out.status.code(), Some(4), "{out:?}");
    assert_matches_schema(&error_document(out), "error.schema.json");
    error_document(out)["error"]["rule"]
        .as_str()
        .unwrap()
        .to_string()
}
