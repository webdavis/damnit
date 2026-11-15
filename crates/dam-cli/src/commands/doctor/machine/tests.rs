use dam_adapters::LuaConfigFile;
use dam_application::PullOutcome;
use dam_application::testing::pulling;

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
    let machine = Machine::new(&file, &store, &launcher);

    assert_eq!(machine.answers().map(<[_]>::len), Some(1));
    assert_eq!(*launcher.launched_with.borrow(), [vec![]]);
    assert!(!marker.exists());
}
