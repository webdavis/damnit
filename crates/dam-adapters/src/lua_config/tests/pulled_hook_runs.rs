use dam_application::{
    EditRequest, FunctionFailure, FunctionFault, Pull, PulledChange, RemoteName,
};
use dam_domain::{Object, Task};

use super::filter_support::{every_field_task, handed, oid};
use super::support::Folder;

const SEEN: &str = "seen";

fn hooked(body: &str) -> Folder {
    let folder = Folder::new();
    let seen = folder.path(SEEN);
    let body = body.replace("SEEN", &format!("{seen:?}"));
    folder.with_hook(
        "task_pulled.lua",
        &format!("return function(task, before, pull)\n{body}\nend\n"),
    )
}

fn pulled(
    folder: &Folder,
    before: Option<&Object>,
    change: PulledChange,
) -> Result<Option<EditRequest>, FunctionFailure> {
    let pull = Pull {
        remote: RemoteName("todoist".into()),
        change,
    };
    folder.loaded().hooks.pulled(
        &handed(&every_field_task()),
        before.map(handed).as_ref(),
        &pull,
    )
}

const WRITE_WHAT_IT_WAS_HANDED: &str = "local file = io.open(SEEN, 'w')\n\
     file:write(task.subject, ' ', before and before.subject or 'nil', ' ', \
     pull.remote, ' ', pull.change)\n\
     file:close()";

#[test]
fn the_pulled_hook_is_handed_the_task_before_and_the_pull() {
    let folder = hooked(WRITE_WHAT_IT_WAS_HANDED);
    let before = Object::Task(Task::new(oid(1), "old report"));
    let ran = pulled(&folder, Some(&before), PulledChange::Changed);
    assert_eq!(ran, Ok(None));
    assert_eq!(folder.text(SEEN), "report old report todoist changed");
}

#[test]
fn a_new_task_is_handed_no_before() {
    let folder = hooked(WRITE_WHAT_IT_WAS_HANDED);
    assert_eq!(pulled(&folder, None, PulledChange::New), Ok(None));
    assert_eq!(folder.text(SEEN), "report nil todoist new");
}

#[test]
fn what_the_pulled_hook_starts_carries_its_name() {
    let folder = hooked("os.execute('printf %s \"$DAM_HOOK\" > ' .. SEEN)");
    pulled(&folder, None, PulledChange::Done).unwrap();
    assert_eq!(folder.text(SEEN), "task_pulled");
}

#[test]
fn the_pulled_hook_returns_an_edit_request_or_fails_like_the_others() {
    let folder = hooked("return { due = 'tomorrow' }");
    let request = pulled(&folder, None, PulledChange::New).unwrap().unwrap();
    assert_eq!(request.json, r#"{"due":"tomorrow"}"#);
    assert_eq!(request.line, Some(1));
    let folder = hooked("error('no bell')");
    let failure = pulled(&folder, None, PulledChange::New).unwrap_err();
    assert_eq!(failure.line, Some(2));
    assert_eq!(
        failure.fault,
        FunctionFault::Raised {
            message: "no bell".into()
        }
    );
}
