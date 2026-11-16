use super::{context, folders, here, subjects_and_paths};
use crate::args::InitArgs;
use crate::commands::init::run;

const WITH_SECTIONS: &str = "return { path = 'damnit/', sections = { 'bugs', 'features' } }";

#[test]
fn in_a_clone_init_creates_the_projects_task_and_each_section() {
    let (ctx, folders) = (context(), folders());
    std::fs::write(folders.project.join(".dam.lua"), WITH_SECTIONS).unwrap();
    let report = run(
        &ctx,
        InitArgs { path: None },
        here(&folders, &folders.project),
    )
    .unwrap();
    let mut made = subjects_and_paths(&ctx);
    made.sort();
    assert_eq!(
        made,
        [
            ("bugs".into(), "damnit/".into()),
            ("damnit".into(), "".into()),
            ("features".into(), "damnit/".into()),
        ]
    );
    assert_eq!(report.data["created"], false);
    assert_eq!(report.data["path"], "damnit/");
    assert!(report.data["task"].is_string());
    assert_eq!(report.data["sections_created"][0]["name"], "bugs");
    assert!(report.data["sections_created"][1]["task"].is_string());
    assert!(
        report.human.contains("created section features as task"),
        "{}",
        report.human
    );
}

#[test]
fn in_a_clone_with_nothing_missing_init_says_nothing_changed() {
    let (ctx, folders) = (context(), folders());
    std::fs::write(folders.project.join(".dam.lua"), WITH_SECTIONS).unwrap();
    let at = || here(&folders, &folders.project);
    run(&ctx, InitArgs { path: None }, at()).unwrap();
    let again = run(&ctx, InitArgs { path: None }, at()).unwrap();
    assert_eq!(subjects_and_paths(&ctx).len(), 3);
    assert!(again.data["path"].is_null());
    assert_eq!(again.data["sections_created"], serde_json::json!([]));
    assert!(again.human.contains("nothing changed"), "{}", again.human);
}

#[test]
fn in_a_clone_whose_file_does_not_load_init_refuses() {
    let (ctx, folders) = (context(), folders());
    std::fs::write(folders.project.join(".dam.lua"), "return { path = 1 }").unwrap();
    let refused = run(
        &ctx,
        InitArgs { path: None },
        here(&folders, &folders.project),
    );
    assert!(refused.is_err());
    assert!(subjects_and_paths(&ctx).is_empty());
}
