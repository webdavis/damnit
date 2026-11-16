use dam_adapters::LuaConfigFile;
use dam_application::testing::{FakeProgress, Said, pulling};
use dam_application::{
    HelperError, HelperLauncher, NoProgress, PullOutcome, RemoteConfig, RemoteHelper, Secret,
};

use super::Machine;

#[test]
fn the_helper_is_asked_with_no_credential_and_no_credential_function_runs() {
    let dir = tempfile::tempdir().unwrap();
    let marker = dir.path().join("called");
    std::fs::write(
        dir.path().join("config.lua"),
        "return { remote = { todoist = { url = 'todoist::' } } }",
    )
    .unwrap();
    std::fs::write(
        dir.path().join("secrets.lua"),
        format!(
            "return {{ remote = {{ todoist = {{ api_token = function() io.open({:?}, 'w'):write('x') return 'tok' end }} }} }}",
            marker.display().to_string()
        ),
    )
    .unwrap();
    let file = LuaConfigFile::new(dir.path().join("config.lua"));
    let launcher = pulling(PullOutcome::default());
    let store = dir.path().join("store");
    let machine = Machine::new(&file, &store, &launcher, &NoProgress);

    assert_eq!(machine.answers().map(<[_]>::len), Some(1));
    assert_eq!(*launcher.launched_with.borrow(), [vec![]]);
    assert!(!marker.exists());
}

fn asked_of_todoist(launcher: &dyn HelperLauncher) -> Vec<Said> {
    let dir = tempfile::tempdir().unwrap();
    let config = "return { remote = { todoist = { url = 'todoist::' } } }";
    std::fs::write(dir.path().join("config.lua"), config).unwrap();
    std::fs::write(dir.path().join("secrets.lua"), "return {}").unwrap();
    let file = LuaConfigFile::new(dir.path().join("config.lua"));
    let progress = FakeProgress::default();
    let store = dir.path().join("store");
    Machine::new(&file, &store, launcher, &progress).answers();
    progress.said.take()
}

fn checking_todoist() -> Said {
    Said::Started("check@todoist: checking todoist ".into(), None)
}

#[test]
fn asking_a_helper_is_a_step_named_after_its_remote_that_finishes() {
    let said = asked_of_todoist(&pulling(PullOutcome::default()));
    assert_eq!(said, [checking_todoist(), Said::Finished]);
}

#[test]
fn a_helper_that_does_not_answer_ends_its_step_as_failed() {
    let said = asked_of_todoist(&NotInstalled);
    assert_eq!(said, [checking_todoist(), Said::Failed]);
}

struct NotInstalled;

impl HelperLauncher for NotInstalled {
    fn launch(
        &self,
        _: &RemoteConfig,
        _: &[(String, Secret)],
    ) -> Result<Box<dyn RemoteHelper>, HelperError> {
        Err(HelperError::NotFound {
            helper: "dam-remote-todoist".into(),
        })
    }
}
