use std::path::{Path as FsPath, PathBuf};

use dam_adapters::read_project_file;
use dam_application::ProjectSettings;
use dam_domain::{Object, Path};
use tempfile::TempDir;

use super::{Here, run};
use crate::args::InitArgs;
use crate::error::CliError;
use crate::testing::context;

struct Folders {
    _root: TempDir,
    project: PathBuf,
    config: PathBuf,
}

fn folders() -> Folders {
    let root = tempfile::tempdir().unwrap();
    let project = root.path().join("damnit");
    let config = root.path().join("config");
    std::fs::create_dir_all(&project).unwrap();
    std::fs::create_dir_all(&config).unwrap();
    Folders {
        _root: root,
        project,
        config,
    }
}

fn here<'a>(folders: &'a Folders, folder: &'a FsPath) -> Here<'a> {
    Here {
        folder,
        personal_config: &folders.config,
    }
}

fn subjects_and_paths(ctx: &crate::context::Context) -> Vec<(String, String)> {
    ctx.store
        .all()
        .unwrap()
        .iter()
        .map(Object::base)
        .map(|base| (base.subject.clone(), base.path.as_str().to_string()))
        .collect()
}

#[test]
fn a_folder_becomes_a_project_named_after_itself() {
    let (ctx, folders) = (context(), folders());
    let report = run(
        &ctx,
        InitArgs { path: None },
        here(&folders, &folders.project),
    )
    .unwrap();
    assert_eq!(
        read_project_file(&folders.project.join(".dam.lua"), "0.2.0"),
        Ok(ProjectSettings {
            path: "damnit/".into()
        })
    );
    assert_eq!(subjects_and_paths(&ctx), [("damnit".into(), "".into())]);
    assert_eq!(report.data["path"], "damnit/");
    assert_eq!(report.data["created"], true);
}

#[test]
fn the_path_flag_sets_where_the_projects_tasks_go() {
    let (ctx, folders) = (context(), folders());
    let path = Some(Path::parse("work/app").unwrap());
    run(&ctx, InitArgs { path }, here(&folders, &folders.project)).unwrap();
    assert_eq!(
        read_project_file(&folders.project.join(".dam.lua"), "0.2.0")
            .unwrap()
            .path,
        "work/app/"
    );
    assert_eq!(subjects_and_paths(&ctx), [("app".into(), "work/".into())]);
}

fn trash_project_file(folder: &FsPath) {
    std::fs::remove_file(folder.join(".dam.lua")).unwrap();
}

#[test]
fn a_second_init_changes_nothing_and_says_so() {
    let (ctx, folders) = (context(), folders());
    let file = folders.project.join(".dam.lua");
    std::fs::write(&file, "return { path = 'kept/' }").unwrap();
    let report = run(
        &ctx,
        InitArgs { path: None },
        here(&folders, &folders.project),
    )
    .unwrap();
    assert_eq!(
        std::fs::read_to_string(&file).unwrap(),
        "return { path = 'kept/' }"
    );
    assert!(subjects_and_paths(&ctx).is_empty());
    assert_eq!(report.data["created"], false);
    assert_eq!(
        report.human,
        format!(
            "{} already makes this folder a project; nothing changed",
            file.display()
        )
    );
}

#[test]
fn a_folder_inside_a_project_becomes_its_own_project() {
    let (ctx, folders) = (context(), folders());
    std::fs::write(folders.project.join(".dam.lua"), "return { path = 'a/' }").unwrap();
    let inner = folders.project.join("src");
    std::fs::create_dir_all(&inner).unwrap();
    let report = run(&ctx, InitArgs { path: None }, here(&folders, &inner)).unwrap();
    assert_eq!(
        read_project_file(&inner.join(".dam.lua"), "0.2.0"),
        Ok(ProjectSettings {
            path: "src/".into()
        })
    );
    assert_eq!(report.data["path"], "src/");
    assert_eq!(
        read_project_file(&folders.project.join(".dam.lua"), "0.2.0"),
        Ok(ProjectSettings { path: "a/".into() })
    );
}

#[test]
fn a_folder_whose_name_is_no_path_is_refused_asking_for_one() {
    let (ctx, folders) = (context(), folders());
    let refusal = run(
        &ctx,
        InitArgs { path: None },
        here(&folders, FsPath::new("/")),
    )
    .unwrap_err();
    assert!(matches!(refusal, CliError::Usage(text) if text.contains("--path")));
}

#[test]
fn the_personal_config_folder_is_refused_because_it_is_never_a_project() {
    let (ctx, folders) = (context(), folders());
    let own = folders.config.clone();
    let refusal = run(&ctx, InitArgs { path: None }, here(&folders, &own)).unwrap_err();
    assert_eq!(
        refusal.to_string(),
        format!(
            "{} holds your own config, which is never a project; run dam init in a project's folder",
            own.display()
        )
    );
    assert!(!own.join(".dam.lua").exists());
}

mod joining;
