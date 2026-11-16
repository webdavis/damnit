use dam_application::testing::{allowed, holder};
use dam_domain::Permission::{TaskChange, TaskRead};

use super::confirm::{asking_above, subject};
use super::*;
use crate::piped::with_piped_ids;

pub(super) fn piped(ctx: &Context, words: &[&str], ids: &[String]) -> Result<Report, CliError> {
    let rows: String = ids
        .iter()
        .map(|id| format!("{id}  p4  -  -  owner  t\n"))
        .collect();
    let typed: Vec<String> = words.iter().map(|w| w.to_string()).collect();
    let line = std::iter::once("dam").chain(words.iter().copied());
    let cli = Cli::try_parse_from(line).map_err(|e| CliError::Usage(e.to_string()))?;
    let cli = with_piped_ids(cli, &mut rows.as_bytes(), false, &typed)?.expect("ids came in");
    dispatch_a_verb_that_opens_the_store(ctx, cli.command)
}

#[test]
fn one_piped_id_is_a_set_and_answers_in_the_set_shape() {
    let (ctx, oids) = three_tasks();
    for words in [
        &["done", "-"][..],
        &["edit", "-", "--subject", "x"],
        &["mv", "-", "garden"],
        &["rm", "-"],
    ] {
        let out = piped(&ctx, words, &oids[..1]).unwrap();
        assert_eq!(out.data["count"], 1, "{words:?}");
        assert!(out.data["changes"].is_array(), "{words:?}");
    }
}

#[test]
fn piped_ids_above_confirm_above_are_refused_with_the_command_and_count() {
    let (ctx, oids) = asking_above(2, &["y"]);
    let refused = piped(&ctx, &["edit", "-", "--subject", "x"], &oids).unwrap_err();
    assert_eq!(refused.exit_code(), 2);
    assert_eq!(
        refused.to_string(),
        "3 ids came in on standard input, more than confirm_above, and a pipe cannot answer \
the question; end the same pipe with: dam edit - --subject x --expect 3"
    );
    assert_eq!(subject(&ctx, &oids[0]), "t1");
}

#[test]
fn piped_ids_at_or_below_confirm_above_are_not_asked() {
    let (ctx, oids) = asking_above(3, &[]);
    let out = piped(&ctx, &["edit", "-", "--subject", "x"], &oids).unwrap();
    assert_eq!(last_line(&out), "changed 3 of 3");
}

#[test]
fn expect_checks_and_preview_shows_piped_ids() {
    let (ctx, oids) = asking_above(1, &[]);
    let refused = piped(
        &ctx,
        &["edit", "-", "--subject", "x", "--expect", "2"],
        &oids,
    );
    assert!(refused.is_err());
    let shown = piped(&ctx, &["done", "-", "--preview"], &oids).unwrap();
    assert_eq!(shown.data["preview"], true);
    let out = piped(
        &ctx,
        &["edit", "-", "--subject", "x", "--expect", "3"],
        &oids,
    )
    .unwrap();
    assert_eq!(last_line(&out), "changed 3 of 3");
    assert_eq!(subject(&ctx, &oids[2]), "x");
}

#[test]
fn a_token_pipes_ids_as_one_set() {
    let (mut ctx, oids) = three_tasks();
    ctx.access = crate::testing::holding(&[TaskRead, TaskChange]);
    let out = piped(&ctx, &["edit", "-", "--subject", "x", "--as-one"], &oids).unwrap();
    assert!(out.data["set"].is_string());
    assert_eq!(out.data["changes"].as_array().unwrap().len(), 3);
}

#[test]
fn a_token_that_may_change_part_changes_nothing_and_names_only_what_came_in() {
    let (mut ctx, oids) = three_tasks();
    ctx.access = Box::new(holder(vec![allowed(
        &[TaskRead, TaskChange],
        Some(&["work"]),
    )]));
    let refused = piped(&ctx, &["edit", "-", "--subject", "x"], &oids).unwrap_err();
    assert!(refused.to_string().contains(&oids[2][..7]), "{refused}");
    assert!(!refused.to_string().contains(&oids[0][..7]), "{refused}");
    assert_eq!(subject(&ctx, &oids[0]), "t1");
}
