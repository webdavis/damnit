use clap::Parser;
use dam_application::{Holder, Owner};
use dam_domain::{Grant, Grants, Path, Token};

use super::*;
use crate::args::Cli;

fn command(line: &str) -> Command {
    Cli::try_parse_from(std::iter::once("dam").chain(line.split_whitespace()))
        .unwrap()
        .command
}

fn holding(grants: Vec<Grant>) -> Holder {
    Holder {
        name: "vpt".into(),
        grants: Grants {
            holder: Token::generate(&mut |b: &mut [u8]| b.fill(1))
                .key_id()
                .clone(),
            grants,
        },
        channel: dam_domain::Channel::Local,
    }
}

fn allowing(permissions: &[Permission]) -> Holder {
    holding(vec![Grant {
        allow: permissions.iter().copied().collect(),
        ..Grant::default()
    }])
}

const EVERY_COMMAND: [&str; 25] = [
    "new x",
    "done abcd",
    "edit abcd --subject y",
    "mv abcd work",
    "rm abcd",
    "add -A",
    "reset",
    "restore abcd",
    "commit -m m",
    "log",
    "show abcd",
    "status",
    "diff",
    "ls",
    "agenda",
    "category list",
    "filter list",
    "remote add r r::",
    "remote list",
    "push",
    "pull",
    "day show",
    "day set",
    "day list",
    "reminders",
];

#[test]
fn the_owner_is_admitted_to_every_command() {
    for line in EVERY_COMMAND.iter().chain(&["resolve abcd --ours"]) {
        assert_eq!(admit(&Owner, &command(line)), Ok(()), "{line}");
    }
}

#[test]
fn a_token_holding_every_permission_still_cannot_write_the_owners_layers() {
    let everything = allowing(&Permission::ALL);
    for verb in ["mv", "rm", "add", "reset", "restore", "commit"] {
        let line = EVERY_COMMAND.iter().find(|l| l.starts_with(verb)).unwrap();
        assert_eq!(
            admit(&everything, &command(line)),
            Err(Refusal::NotOpenToTokens(verb))
        );
    }
    assert_eq!(
        admit(&everything, &command("remote add r r::")),
        Err(Refusal::NotOpenToTokens("remote add"))
    );
    assert_eq!(
        admit(&everything, &command("resolve abcd --ours")),
        Err(Refusal::NotOpenToTokens("resolve"))
    );
    for line in ["day show", "day set", "day list"] {
        assert_eq!(
            admit(&everything, &command(line)),
            Err(Refusal::NotOpenToTokens("day"))
        );
    }
}

#[test]
fn new_edit_and_done_send_items_for_a_token_holding_their_permission() {
    let reader = allowing(&[Permission::TaskRead]);
    for (line, permissions) in [
        (
            "new milk",
            &[Permission::TaskCreate, Permission::EventCreate][..],
        ),
        (
            "edit abcd --subject x",
            &[Permission::TaskChange, Permission::EventChange],
        ),
        ("done abcd", &[Permission::TaskComplete]),
    ] {
        for permission in permissions {
            assert_eq!(admit(&allowing(&[*permission]), &command(line)), Ok(()));
        }
        assert_eq!(
            admit(&reader, &command(line)),
            Err(Refusal::NotPermitted(permissions[0]))
        );
    }
}

#[test]
fn a_token_holding_nothing_is_refused_every_command_but_status() {
    let nothing = allowing(&[]);
    for line in EVERY_COMMAND.iter().chain(&["resolve abcd --ours"]) {
        let admitted = admit(&nothing, &command(line));
        assert_eq!(admitted.is_ok(), *line == "status", "{line}");
    }
}

#[test]
fn whole_store_reads_need_every_permission_they_name_without_restrictions() {
    let status = command("diff");
    assert_eq!(
        admit(&allowing(&[Permission::WorkingRead]), &status),
        Err(Refusal::NotPermitted(Permission::StageRead))
    );
    assert_eq!(
        admit(
            &allowing(&[Permission::WorkingRead, Permission::StageRead]),
            &status
        ),
        Ok(())
    );
    let narrowed = holding(vec![Grant {
        allow: [Permission::HistoryRead].into(),
        paths: Some(vec![Path::parse("work").unwrap()]),
        ..Grant::default()
    }]);
    assert_eq!(
        admit(&narrowed, &command("log")),
        Err(Refusal::NotPermitted(Permission::HistoryRead))
    );
}

#[test]
fn narrowed_reads_need_one_of_their_permissions_in_any_grant() {
    let events = allowing(&[Permission::EventRead]);
    assert_eq!(admit(&events, &command("ls")), Ok(()));
    assert_eq!(admit(&events, &command("agenda")), Ok(()));
    assert_eq!(admit(&events, &command("reminders")), Ok(()));
    assert_eq!(
        admit(&allowing(&[Permission::TaskRead]), &command("agenda")),
        Err(Refusal::NotPermitted(Permission::EventRead))
    );
    assert_eq!(
        admit(&allowing(&[Permission::RemotePull]), &command("push")),
        Err(Refusal::NotPermitted(Permission::RemotePush))
    );
}
