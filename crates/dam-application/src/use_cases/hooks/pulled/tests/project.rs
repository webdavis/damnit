use dam_domain::{Object, Path};

use super::*;
use crate::service::{Dam, ProjectHooks};
use crate::testing::FakeHooks;
use crate::testing::task_in;

fn incoming_in(subject: &str, path: &str, remote_id: &str) -> IncomingObject {
    IncomingObject {
        remote_id: remote_id.to_string(),
        object: Object::Task(task_in(0, subject, path)),
    }
}

fn pulled_with_project(
    s: &Scenario,
    hooks: &FakeHooks,
    objects: Vec<IncomingObject>,
) -> PullReport {
    let project_config = Config {
        task_pulled: Some(PulledHook::default()),
        ..config()
    };
    let within = Path::parse("app/").unwrap();
    let dam = Dam {
        project_hooks: Some(ProjectHooks {
            config: &project_config,
            hooks,
            within: &within,
        }),
        ..s.dam()
    };
    let outcome = PullOutcome {
        objects,
        sync: Some("s1".into()),
        ..PullOutcome::default()
    };
    let l = pulling(outcome);
    dam.pull(remotes_launched_by(&l), None).unwrap().remove(0)
}

fn subjects(hooks: &FakeHooks) -> Vec<String> {
    hooks
        .pulled
        .borrow()
        .iter()
        .map(|(task, _, _)| task.clone())
        .collect()
}

#[test]
fn the_projects_pulled_hook_sees_only_tasks_under_the_project() {
    let s = pulled_hooked();
    let project = FakeHooks::default();
    let report = pulled_with_project(
        &s,
        &project,
        vec![
            incoming_in("fix the build", "app/", "r1"),
            incoming_in("buy milk", "home/", "r2"),
        ],
    );
    assert_eq!(calls(&s).len(), 2);
    assert_eq!(subjects(&project), ["fix the build"]);
    let run = report.task_pulled.unwrap();
    assert_eq!((run.ran, run.failed, run.not_reached), (3, 0, 0));
}

#[test]
fn the_projects_pulled_hook_runs_without_a_personal_one() {
    let s = Scenario::default().drawing_from(42).configured(config());
    s.store.set_sync_token(&todoist(), Some("s0")).unwrap();
    let project = FakeHooks::default();
    let report = pulled_with_project(&s, &project, vec![incoming_in("fix", "app/", "r1")]);
    assert_eq!(subjects(&project), ["fix"]);
    assert_eq!(report.task_pulled.unwrap().ran, 1);
}
