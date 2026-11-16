use dam_domain::Path;

use super::hooked;
use crate::config::{Config, HookConfig, HookEvent};
use crate::errors::{Refusal, UseCaseError};
use crate::service::{Dam, ProjectHooks};
use crate::testing::{FakeHooks, Scenario, config};
use crate::use_cases::hooks::listed::ListedHookRun;

fn project_config(declared: bool) -> Config {
    let hooks = declared.then(|| HookConfig {
        event: HookEvent::TaskNew,
        on_failure: None,
        file: ".damhooks/task_new.lua".into(),
    });
    Config {
        hooks: hooks.into_iter().collect(),
        ..config()
    }
}

fn run_with(
    s: &Scenario,
    project: &Config,
    hooks: &FakeHooks,
) -> Result<ListedHookRun, UseCaseError> {
    let within = Path::parse("home/").unwrap();
    let dam = Dam {
        project_hooks: Some(ProjectHooks {
            config: project,
            hooks,
            within: &within,
        }),
        ..s.dam()
    };
    dam.run_hook_over_listed(HookEvent::TaskNew, None)
}

#[test]
fn a_projects_hook_runs_only_over_listed_tasks_under_the_project() {
    let s = hooked(&[(HookEvent::TaskNew, None)], None);
    let theirs = FakeHooks::default();
    let run = run_with(&s, &project_config(true), &theirs).unwrap();
    assert_eq!(s.fixture.hooks.ran().len(), 3);
    let subjects: Vec<String> = theirs.ran().into_iter().map(|(_, s)| s).collect();
    assert_eq!(subjects, ["call Ann", "pay rent"]);
    assert_eq!((run.ran, run.not_reached), (5, 0));
}

#[test]
fn a_projects_hook_runs_with_no_personal_one() {
    let s = hooked(&[], None);
    let theirs = FakeHooks::default();
    let run = run_with(&s, &project_config(true), &theirs).unwrap();
    assert_eq!(run.ran, 2);
    assert!(s.fixture.hooks.ran().is_empty());
}

#[test]
fn neither_side_declaring_the_hook_is_still_refused() {
    let s = hooked(&[], None);
    assert_eq!(
        run_with(&s, &project_config(false), &FakeHooks::default()),
        Err(UseCaseError::Refused(Refusal::HookNotDeclared(
            HookEvent::TaskNew
        )))
    );
}
