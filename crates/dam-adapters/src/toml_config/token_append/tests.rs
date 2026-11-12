use std::collections::BTreeSet;

use dam_application::{NewToken, Standing, TokenName};
use dam_domain::{Channel, Grant, Path, Permission, Token};

use super::append_token;
use crate::toml_config::load_config;

fn new_token() -> NewToken {
    NewToken {
        name: TokenName::parse("vpt").unwrap(),
        value: Token::generate(&mut |b: &mut [u8]| b.fill(7)),
        expires: Some("2999-05-11T00:00:00Z".parse().unwrap()),
        via: Some(BTreeSet::from([Channel::Local, Channel::Mcp])),
        require: vec!["session".into()],
        grant: Grant {
            allow: BTreeSet::from([Permission::TaskRead]),
            paths: Some(vec![Path::parse("work").unwrap()]),
            labels: Some(BTreeSet::from(["home".to_string()])),
            own: true,
            remotes: Some(vec!["todoist".into()]),
        },
    }
}

#[test]
fn the_new_token_reads_back_with_every_term_after_the_text_already_there() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("config.toml");
    let before = "# mine\n[done]\ninteractive = true # keep\n";
    std::fs::write(&path, before).unwrap();
    let token = new_token();
    append_token(&path, &token).unwrap();
    assert!(std::fs::read_to_string(&path).unwrap().starts_with(before));
    let config = load_config(&path).unwrap();
    let entry = &config.tokens.entries[0];
    assert_eq!(entry.value, token.value);
    let Standing::Usable(terms) = &entry.standing else {
        panic!("{entry:?}");
    };
    assert_eq!(terms.expires, token.expires);
    assert_eq!(Some(&terms.via), token.via.as_ref());
    assert_eq!(terms.require, token.require);
    assert_eq!(terms.grants, vec![token.grant]);
}

#[test]
fn a_config_that_cannot_take_the_token_is_left_as_it_was() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("config.toml");
    std::fs::write(&path, "token = \"oops\"\n").unwrap();
    assert!(append_token(&path, &new_token()).is_err());
    assert_eq!(
        std::fs::read_to_string(&path).unwrap(),
        "token = \"oops\"\n"
    );
}
