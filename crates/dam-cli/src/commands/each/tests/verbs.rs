use super::piped::piped;
use super::*;

#[test]
fn edit_on_many_oids_answers_one_changes_document() {
    let (ctx, oids) = three_tasks();
    let out = dam(&ctx, &["edit", &oids[0], &oids[1], "--subject", "x"]).unwrap();
    assert_eq!(last_line(&out), "changed 2 of 2");
    assert_eq!(out.data["count"], 2);
    assert_eq!(out.data["preview"], false);
    assert_eq!(out.data["changes"].as_array().unwrap().len(), 2);
    assert!(out.human.contains("+x"));
}

#[test]
fn a_preview_on_one_oid_answers_the_document_and_writes_nothing() {
    let (ctx, oids) = three_tasks();
    let out = dam(&ctx, &["edit", &oids[0], "--subject", "x", "--preview"]).unwrap();
    assert_eq!(
        last_line(&out),
        "preview: 1 of 1 would change, nothing written"
    );
    assert_eq!(out.data["preview"], true);
    let oid = Oid::parse(&oids[0]).unwrap();
    assert_eq!(ctx.store.get(&oid).unwrap().unwrap().base().subject, "t1");
}

#[test]
fn an_empty_set_prints_nothing_matches() {
    let (ctx, _) = three_tasks();
    let out = dam(&ctx, &["done", "--filter", "garden"]).unwrap();
    assert_eq!(out.human.trim(), "nothing matches");
    assert_eq!(out.data["count"], 0);
}

#[test]
fn oids_and_a_filter_together_are_a_usage_problem() {
    let (ctx, oids) = three_tasks();
    let refused = dam(
        &ctx,
        &["edit", &oids[0], "--filter", "work", "--subject", "y"],
    );
    assert!(matches!(refused, Err(CliError::Usage(_))));
}

#[test]
fn edit_e_on_a_set_is_a_usage_problem() {
    let (ctx, oids) = three_tasks();
    let refused = dam(&ctx, &["edit", "-e", &oids[0], &oids[1]]);
    assert!(matches!(refused, Err(CliError::Usage(_))));
}

#[test]
fn done_on_piped_ids_completes_them() {
    let (ctx, oids) = three_tasks();
    let out = piped(&ctx, &["done", "-", "--expect", "2"], &oids[..2]).unwrap();
    assert_eq!(last_line(&out), "changed 2 of 2");
}

#[test]
fn mv_reads_the_last_word_as_the_path() {
    let (ctx, oids) = three_tasks();
    let out = dam(&ctx, &["mv", &oids[0], &oids[2], "garden"]).unwrap();
    assert_eq!(out.data["count"], 2);
    let moved = ctx
        .store
        .get(&Oid::parse(&oids[2]).unwrap())
        .unwrap()
        .unwrap();
    assert_eq!(moved.base().path.as_str(), "garden/");
}

#[test]
fn mv_on_piped_ids_takes_its_path_from_the_path_flag() {
    let (ctx, oids) = three_tasks();
    let out = piped(&ctx, &["mv", "-", "--path", "garden"], &oids[..2]).unwrap();
    assert_eq!(last_line(&out), "changed 2 of 2");
}

#[test]
fn rm_on_many_oids_removes_them_all() {
    let (ctx, oids) = three_tasks();
    let out = dam(&ctx, &["rm", &oids[0], &oids[1]]).unwrap();
    assert_eq!(last_line(&out), "changed 2 of 2");
    assert!(
        ctx.store
            .get(&Oid::parse(&oids[0]).unwrap())
            .unwrap()
            .is_none()
    );
}

#[test]
fn an_expect_that_differs_refuses() {
    let (ctx, oids) = three_tasks();
    let refused = piped(&ctx, &["rm", "-", "--expect", "3"], &oids[..2]);
    assert!(matches!(refused, Err(CliError::UseCase(_))));
}

#[test]
fn mv_refuses_a_relative_segment_in_its_path_word_by_its_name() {
    let (ctx, oids) = three_tasks();
    let Err(CliError::Usage(said)) = dam(&ctx, &["mv", &oids[0], "../x"]) else {
        panic!("a relative path was taken")
    };
    assert!(said.contains("cannot be \".\" or \"..\""), "{said}");
    assert!(!said.contains("Segment"), "{said}");
}
