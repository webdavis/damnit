use dam_domain::{Channel, Grant, Permission, Timestamp, Token};

use super::*;
use crate::tokens::{Standing, TokenConfig, TokenTerms, Tokens};

fn token(fill: u8) -> Token {
    Token::generate(&mut |b: &mut [u8]| b.fill(fill))
}

fn at(text: &str) -> Timestamp {
    text.parse().unwrap()
}

fn terms() -> TokenTerms {
    TokenTerms {
        expires: None,
        not_before: None,
        via: [Channel::Local].into(),
        require: vec![],
        grants: vec![Grant {
            allow: [Permission::TaskRead].into(),
            ..Grant::default()
        }],
    }
}

fn one(name: &str, value: Token, standing: Standing) -> Tokens {
    Tokens {
        names: vec![name.into()],
        entries: vec![TokenConfig {
            name: name.into(),
            value,
            standing,
        }],
        warnings: vec![],
    }
}

const NOW: &str = "2026-11-11T12:00:00Z";

fn local(tokens: &Tokens, presented: &str) -> Result<Holder, TokenRefusal> {
    authenticate(
        tokens,
        presented,
        Channel::Local,
        at(NOW),
        &Default::default(),
    )
}

#[test]
fn a_known_usable_token_names_its_holder_and_carries_its_grants() {
    let tokens = one("tool", token(1), Standing::Usable(terms()));
    let holder = local(&tokens, &token(1).expose()).unwrap();
    assert_eq!(holder.name, "tool");
    assert_eq!(&holder.grants.holder, token(1).key_id());
    assert_eq!(holder.grants.grants, terms().grants);
}

#[test]
fn text_that_is_not_a_well_formed_token_is_malformed() {
    let tokens = one("tool", token(1), Standing::Usable(terms()));
    let mut tampered = token(1).expose();
    let swapped = if &tampered[10..11] == "a" { "b" } else { "a" };
    tampered.replace_range(10..11, swapped);
    for presented in ["", "hunter2", "damk_v1_short", tampered.as_str()] {
        assert_eq!(local(&tokens, presented), Err(TokenRefusal::Malformed));
    }
}

#[test]
fn a_version_this_dam_does_not_know_is_named() {
    let tokens = one("tool", token(1), Standing::Usable(terms()));
    assert_eq!(
        local(&tokens, "damk_v9_abc"),
        Err(TokenRefusal::UnknownVersion("v9".into()))
    );
}

#[test]
fn a_well_formed_token_no_config_entry_holds_is_unknown() {
    let tokens = one("tool", token(1), Standing::Usable(terms()));
    assert_eq!(
        local(&tokens, &token(2).expose()),
        Err(TokenRefusal::Unknown)
    );
}

#[test]
fn a_disabled_token_is_refused_with_its_reason() {
    let tokens = one("tool", token(1), Standing::Disabled("unknown key x".into()));
    assert_eq!(
        local(&tokens, &token(1).expose()),
        Err(TokenRefusal::Disabled {
            name: "tool".into(),
            why: "unknown key x".into()
        })
    );
}

#[test]
fn a_token_is_refused_before_not_before_and_from_expires_on() {
    let mut t = terms();
    t.not_before = Some(at("2026-11-11T12:00:01Z"));
    let early = one("tool", token(1), Standing::Usable(t));
    assert!(matches!(
        local(&early, &token(1).expose()),
        Err(TokenRefusal::NotYetValid { .. })
    ));
    let mut t = terms();
    t.expires = Some(at(NOW));
    let expired = one("tool", token(1), Standing::Usable(t));
    assert!(matches!(
        local(&expired, &token(1).expose()),
        Err(TokenRefusal::Expired { .. })
    ));
    let mut t = terms();
    t.expires = Some(at("2026-11-11T12:00:01Z"));
    t.not_before = Some(at(NOW));
    assert!(
        local(
            &one("tool", token(1), Standing::Usable(t)),
            &token(1).expose()
        )
        .is_ok()
    );
}

#[test]
fn a_token_is_refused_on_a_channel_its_via_leaves_out() {
    let mut t = terms();
    t.via = [Channel::Mcp].into();
    let tokens = one("tool", token(1), Standing::Usable(t));
    assert_eq!(
        local(&tokens, &token(1).expose()),
        Err(TokenRefusal::ChannelNotAllowed {
            name: "tool".into(),
            channel: Channel::Local
        })
    );
    assert!(
        authenticate(
            &tokens,
            &token(1).expose(),
            Channel::Mcp,
            at(NOW),
            &Default::default()
        )
        .is_ok()
    );
}

#[test]
fn every_refusal_has_its_own_rule_name() {
    let refusals = [
        TokenRefusal::Malformed,
        TokenRefusal::UnknownVersion("v9".into()),
        TokenRefusal::Unknown,
        TokenRefusal::Disabled {
            name: "a".into(),
            why: "b".into(),
        },
        TokenRefusal::NotYetValid { name: "a".into() },
        TokenRefusal::Expired { name: "a".into() },
        TokenRefusal::ChannelNotAllowed {
            name: "a".into(),
            channel: Channel::Local,
        },
    ];
    let names: std::collections::BTreeSet<&str> = refusals.iter().map(|r| r.name()).collect();
    assert_eq!(names.len(), refusals.len());
}

#[test]
fn a_token_that_requires_details_is_refused_naming_only_those_the_request_left_out() {
    let requiring = TokenTerms {
        require: vec!["session".into(), "model".into()],
        ..terms()
    };
    let tokens = one("claude", token(1), Standing::Usable(requiring));
    let offered = [("session".to_string(), "s1".to_string())].into();
    assert_eq!(
        authenticate(
            &tokens,
            &token(1).expose(),
            Channel::Local,
            at(NOW),
            &offered
        ),
        Err(TokenRefusal::MissingDetails {
            name: "claude".into(),
            details: vec!["model".into()],
        })
    );
}
