use super::*;

fn labelled_review(ctx: &Context, subject: &str) -> String {
    let all = ctx.store.all().unwrap();
    let mut task = all
        .into_iter()
        .find(|o| o.base().subject == subject)
        .unwrap();
    task.base_mut().labels.insert("review".into());
    ctx.store.put(&task).unwrap();
    task.oid().as_str().into()
}

fn with_stage() -> Standing {
    inside_with(ProjectSettings {
        categories: stage(),
        ..settings()
    })
}

#[test]
fn edit_refuses_a_second_label_of_a_projects_exclusive_category() {
    let ctx = holding_a_project_and_more();
    let readme = labelled_review(&ctx, "readme");
    let refused = run(ctx, &["edit", &readme, "--labels", "ship"], with_stage());
    assert!(refused.is_err());
    let ctx = holding_a_project_and_more();
    labelled_review(&ctx, "readme");
    let words = ["edit", "--query", "kind:task", "--labels", "ship"];
    assert!(run(ctx, &words, with_stage()).is_err());
}

#[test]
fn edit_outside_the_projects_path_or_global_keeps_the_persons_categories() {
    let ctx = holding_a_project_and_more();
    let laundry = labelled_review(&ctx, "laundry");
    let words = ["edit", &laundry, "--labels", "ship"];
    assert!(run(ctx, &words, with_stage()).is_ok());
    let ctx = holding_a_project_and_more();
    let readme = labelled_review(&ctx, "readme");
    let words = ["edit", &readme, "--labels", "ship", "--global"];
    assert!(run(ctx, &words, with_stage()).is_ok());
}

#[test]
fn edit_judges_by_the_projects_category_under_its_path_and_the_persons_outside() {
    let persons_stage = Categories::new(vec![Category {
        name: "stage".into(),
        values: vec!["review".into(), "ship".into()],
        exclusive: false,
    }])
    .unwrap();
    let ctx = || {
        let mut ctx = holding_a_project_and_more();
        ctx.config.categories = persons_stage.clone();
        ctx
    };
    let under = ctx();
    let readme = labelled_review(&under, "readme");
    assert!(run(under, &["edit", &readme, "--labels", "ship"], with_stage()).is_err());
    let outside = ctx();
    let laundry = labelled_review(&outside, "laundry");
    assert!(
        run(
            outside,
            &["edit", &laundry, "--labels", "ship"],
            with_stage()
        )
        .is_ok()
    );
}
