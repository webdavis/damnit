use dam_application::Standing;
use dam_domain::{Permission, Token};

use super::TEMPLATE;
use crate::toml_config::parse_config;

fn is_example(line: &str) -> bool {
    line.strip_prefix('#')
        .is_some_and(|rest| !rest.is_empty() && !rest.starts_with([' ', '#']))
}

fn uncommented(line: &str) -> &str {
    if is_example(line) { &line[1..] } else { line }
}

fn example_blocks() -> Vec<Vec<&'static str>> {
    let mut blocks: Vec<Vec<&str>> = vec![Vec::new()];
    for line in TEMPLATE.lines() {
        if is_example(line) {
            blocks.last_mut().unwrap().push(&line[1..]);
        } else if !blocks.last().unwrap().is_empty() {
            blocks.push(Vec::new());
        }
    }
    blocks.retain(|b| !b.is_empty());
    blocks
}

#[test]
fn the_template_reads_as_an_empty_config() {
    let config = parse_config(TEMPLATE).unwrap();
    assert!(config.remotes.is_empty());
    assert!(config.tokens.entries.is_empty());
    assert!(
        config.tokens.warnings.is_empty(),
        "{:?}",
        config.tokens.warnings
    );
}

#[test]
fn every_example_block_reads_on_its_own() {
    let blocks = example_blocks();
    assert_eq!(blocks.len(), 6, "{blocks:?}");
    for block in blocks {
        let text = block.join("\n");
        parse_config(&text).unwrap_or_else(|e| panic!("{e:?}:\n{text}"));
    }
}

#[test]
fn every_example_uncommented_at_once_is_a_working_config() {
    let value = Token::generate(&mut |b: &mut [u8]| b.fill(3)).expose();
    let text: String = TEMPLATE
        .lines()
        .map(uncommented)
        .collect::<Vec<_>>()
        .join("\n")
        .replace("damk_v1_...", &value);
    let config = parse_config(&text).unwrap();
    assert_eq!(config.owner_name(), "Ada");
    let remote = config.remote("todoist").unwrap();
    assert_eq!(remote.credentials.len(), 1, "{remote:?}");
    assert!(remote.stale.is_some() && remote.deadline.is_some() && remote.path.is_some());
    assert_eq!(config.filters.len(), 1);
    assert!(
        config.tokens.warnings.is_empty(),
        "{:?}",
        config.tokens.warnings
    );
    let [token] = config.tokens.entries.as_slice() else {
        panic!("{:?}", config.tokens.entries);
    };
    let Standing::Usable(terms) = &token.standing else {
        panic!("{token:?}");
    };
    assert!(terms.expires.is_some() && terms.not_before.is_some());
    let [grant] = terms.grants.as_slice() else {
        panic!("{terms:?}");
    };
    assert_eq!(grant.allow.len(), 3);
    assert!(grant.paths.is_some() && grant.labels.is_some() && grant.remotes.is_some());
}

#[test]
fn every_permission_dam_grants_is_named() {
    for permission in Permission::ALL {
        assert!(TEMPLATE.contains(permission.as_str()), "{permission:?}");
    }
}
