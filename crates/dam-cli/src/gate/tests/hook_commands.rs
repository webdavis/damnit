use dam_application::{Owner, Refusal};
use dam_domain::Permission;

use super::{allowing, command, holding};
use crate::gate::{Need, admit, need, refuse_any_token_to_an_owner_only_verb};

#[test]
fn only_the_owner_lists_or_runs_hooks_whatever_a_token_holds() {
    for (line, verb) in [
        ("hook list", "hook list"),
        ("hook run task_new", "hook run"),
        ("hook run task_pulled --due today", "hook run"),
    ] {
        assert_eq!(admit(&Owner, &command(line)), Ok(()), "{line}");
        for token in [
            holding(vec![]),
            allowing(&[Permission::ConfigRead]),
            allowing(&Permission::ALL),
        ] {
            assert_eq!(
                admit(&token, &command(line)),
                Err(Refusal::NotOpenToTokens(verb))
            );
        }
        assert_eq!(
            refuse_any_token_to_an_owner_only_verb(true, &command(line)),
            Err(Refusal::NotOpenToTokens(verb))
        );
    }
}

#[test]
fn bare_hook_only_prints_its_help() {
    assert_eq!(need(&command("hook")), Need::Open);
}

#[test]
fn no_token_turns_project_hooks_on_or_off_whatever_it_holds() {
    for (line, verb) in [
        ("config set project.hooks true", "config set"),
        ("config set --global project.hooks true", "config set"),
        ("config set ls label:app", "config set"),
        ("config set --global owner Grace", "config set"),
        ("config unset project.hooks", "config unset"),
        ("config unset ls", "config unset"),
    ] {
        assert_eq!(
            refuse_any_token_to_an_owner_only_verb(true, &command(line)),
            Err(Refusal::NotOpenToTokens(verb))
        );
        assert_eq!(
            admit(&allowing(&Permission::ALL), &command(line)),
            Err(Refusal::NotOpenToTokens(verb))
        );
    }
}
