use crate::args::StatusArgs;
use crate::output::fitted;
use crate::testing::context;
use dam_application::{CredentialSpec, Notice, RemoteConfig, RemoteName};
use dam_domain::{Object, Oid, Task};

const WIDTH: usize = 20;

fn oid(b: u8) -> Oid {
    Oid::generate(&mut |x: &mut [u8]| x.fill(b))
}

#[test]
fn status_cuts_its_rows_and_prints_warnings_and_notices_whole() {
    let mut ctx = context();
    ctx.config.remotes = vec![RemoteConfig {
        name: RemoteName("todoist".into()),
        helper: "todoist".into(),
        url: "todoist::".into(),
        credentials: vec![CredentialSpec::Literal {
            name: "api_token".into(),
            value: "secret".into(),
        }],
        stale: None,
        deadline: None,
        path: None,
    }];
    ctx.store
        .put(&Object::Task(Task::new(
            oid(1),
            "a subject longer than the width",
        )))
        .unwrap();
    ctx.store
        .add_notice(&Notice::PushFailed {
            remote: RemoteName("todoist".into()),
            oid: oid(1),
            why: "the helper refused the change for a long reason".into(),
        })
        .unwrap();
    let report = super::run_status(&ctx, StatusArgs { full: false }).unwrap();
    let text = fitted(&report, WIDTH);
    let whole: Vec<&str> = report
        .human
        .lines()
        .filter(|l| l.starts_with("warning:") || l.contains("long reason"))
        .collect();
    assert_eq!(whole.len(), 2, "{}", report.human);
    for line in &whole {
        assert!(text.lines().any(|l| l == *line), "{line} was cut:\n{text}");
    }
    let rows: Vec<&str> = text.lines().filter(|l| !whole.contains(l)).collect();
    assert!(rows.iter().all(|l| l.chars().count() <= WIDTH), "{text}");
    assert!(rows.iter().any(|l| l.ends_with('…')), "{text}");
}
