use crate::remote::{RemoteCapabilities, RemoteMutation};
use crate::testing::prelude::*;
use crate::testing::{
    Pushed, Scenario, ScriptedHelper, ScriptedLauncher, remotes_launched_by, task, task_caps,
    todoist,
};

#[test]
fn a_remote_that_declares_no_kinds_is_sent_nothing_and_owed_nothing() {
    let s = Scenario::default();
    s.committed(task(1));
    let pushed = Pushed::default();
    let seen = pushed.clone();
    let l = ScriptedLauncher::launching(move || ScriptedHelper {
        caps: RemoteCapabilities {
            kinds: vec![],
            ..task_caps()
        },
        push_answer: Box::new(|_: &[RemoteMutation]| {
            panic!("a remote that declares no kinds was sent a push")
        }),
        pushed: seen.clone(),
        ..ScriptedHelper::answering(Ok(Default::default()))
    });
    let reports = s.dam().push(remotes_launched_by(&l), None).unwrap();
    assert_eq!((reports[0].sent, reports[0].skipped), (0, 1));
    assert!(pushed.borrow().is_empty());
    assert!(
        s.store.unpushed(&todoist()).unwrap().is_empty(),
        "nothing stays owed to it"
    );
    assert!(
        s.store.notices().unwrap().is_empty(),
        "nothing was refused, so nothing is reported"
    );
}
