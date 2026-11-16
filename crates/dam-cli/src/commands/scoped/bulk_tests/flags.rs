use super::*;

#[test]
fn a_query_inside_a_project_picks_only_its_tasks() {
    each_verb_picks(
        &["--query", "kind:task"],
        inside,
        &["parse flags", "readme"],
    );
}

#[test]
fn a_path_in_the_query_starts_from_the_project() {
    each_verb_picks(&["--query", "path:cli/"], inside, &["parse flags"]);
}

#[test]
fn a_filter_inside_a_project_picks_only_its_tasks() {
    each_verb_picks(&["--filter", "tasks"], inside, &["parse flags", "readme"]);
}

#[test]
fn the_projects_filter_wins_over_the_persons_of_the_same_name() {
    let project = || {
        inside_with(ProjectSettings {
            filters: vec![tasks_filter("path:cli/")],
            ..settings()
        })
    };
    each_verb_picks(&["--filter", "tasks"], project, &["parse flags"]);
}

#[test]
fn global_picks_from_every_task_with_the_persons_filter() {
    let project = || {
        inside_with(ProjectSettings {
            filters: vec![tasks_filter("path:cli/")],
            ..settings()
        })
    };
    let everything = ["laundry", "other cli", "parse flags", "readme"];
    each_verb_picks(&["--global", "--query", "kind:task"], inside, &everything);
    each_verb_picks(&["--global", "--filter", "tasks"], project, &everything);
}

#[test]
fn project_picks_the_named_projects_tasks() {
    each_verb_picks(
        &["--project", "cli/", "--query", "kind:task"],
        inside,
        &["other cli"],
    );
}

#[test]
fn outside_a_project_a_query_picks_from_every_task() {
    let everything = ["laundry", "other cli", "parse flags", "readme"];
    each_verb_picks(&["--query", "kind:task"], || Standing::Outside, &everything);
}

#[test]
fn oids_from_outside_the_project_are_never_narrowed() {
    for verb in VERBS {
        let ctx = holding_a_project_and_more();
        let laundry = oid_of(&ctx, "laundry");
        let picked = picked(ctx, verb, &[&laundry], inside());
        assert_eq!(picked, ["laundry"], "dam {verb:?}");
    }
}

#[test]
fn a_projects_category_reads_in_the_query() {
    for verb in VERBS {
        let ctx = holding_a_project_and_more();
        let mut readme = ctx.store.all().unwrap()[0].clone();
        readme.base_mut().labels.insert("review".into());
        ctx.store.put(&readme).unwrap();
        let project = inside_with(ProjectSettings {
            categories: stage(),
            ..settings()
        });
        let picked = picked(ctx, verb, &["--query", "stage:review"], project);
        assert_eq!(picked.len(), 1, "dam {verb:?}");
    }
}

#[test]
fn expect_counts_the_projects_tasks() {
    for verb in VERBS {
        let words = [verb, &["--query", "kind:task", "--expect", "2"]].concat();
        assert!(run(holding_a_project_and_more(), &words, inside()).is_ok());
        let words = [verb, &["--query", "kind:task", "--expect", "4"]].concat();
        assert!(run(holding_a_project_and_more(), &words, inside()).is_err());
    }
}

#[test]
fn a_project_file_that_does_not_load_refuses_a_query_but_not_oids() {
    for verb in VERBS {
        let words = [verb, &["--query", "kind:task"]].concat();
        let refused = run(holding_a_project_and_more(), &words, unreadable());
        assert!(matches!(refused, Err(CliError::Config(_))), "dam {verb:?}");
        let words = [verb, &["--global", "--query", "kind:task"]].concat();
        assert!(run(holding_a_project_and_more(), &words, unreadable()).is_ok());
    }
    let ctx = holding_a_project_and_more();
    let laundry = oid_of(&ctx, "laundry");
    assert!(run(ctx, &["done", &laundry], unreadable()).is_ok());
}
