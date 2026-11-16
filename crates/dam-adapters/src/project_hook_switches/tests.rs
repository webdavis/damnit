use dam_application::ProjectHookSwitches;

use super::ProjectHookSwitchFile;

struct Machine {
    dir: tempfile::TempDir,
}

impl Machine {
    fn new() -> Machine {
        let dir = tempfile::tempdir().unwrap();
        for clone in ["first", "second"] {
            std::fs::create_dir(dir.path().join(clone)).unwrap();
        }
        Machine { dir }
    }

    fn switches(&self) -> ProjectHookSwitchFile {
        ProjectHookSwitchFile::beside_the_store(&self.dir.path().join("state").join("dam.db"))
    }

    fn clone_at(&self, name: &str) -> std::path::PathBuf {
        self.dir.path().join(name)
    }
}

#[test]
fn a_clone_never_switched_is_unset() {
    let machine = Machine::new();
    assert_eq!(
        machine.switches().switch(&machine.clone_at("first")),
        Ok(None)
    );
}

#[test]
fn a_switch_is_kept_for_its_own_clone_only() {
    let machine = Machine::new();
    machine
        .switches()
        .turn(&machine.clone_at("first"), Some(true))
        .unwrap();
    machine
        .switches()
        .turn(&machine.clone_at("second"), Some(false))
        .unwrap();
    assert_eq!(
        machine.switches().switch(&machine.clone_at("first")),
        Ok(Some(true))
    );
    assert_eq!(
        machine.switches().switch(&machine.clone_at("second")),
        Ok(Some(false))
    );
}

#[test]
fn a_clone_is_named_by_its_canonical_folder() {
    let machine = Machine::new();
    let roundabout = machine.clone_at("second").join("..").join("first");
    machine.switches().turn(&roundabout, Some(true)).unwrap();
    assert_eq!(
        machine.switches().switch(&machine.clone_at("first")),
        Ok(Some(true))
    );
}

#[test]
fn unsetting_forgets_the_clone() {
    let machine = Machine::new();
    let first = machine.clone_at("first");
    machine.switches().turn(&first, Some(true)).unwrap();
    machine.switches().turn(&first, None).unwrap();
    assert_eq!(machine.switches().switch(&first), Ok(None));
}

#[test]
fn the_record_sits_beside_the_store_outside_any_project() {
    let machine = Machine::new();
    machine
        .switches()
        .turn(&machine.clone_at("first"), Some(true))
        .unwrap();
    let kept = machine.dir.path().join("state").join("project-hooks.json");
    let canonical = std::fs::canonicalize(machine.clone_at("first")).unwrap();
    let expected = serde_json::json!({ canonical.to_str().unwrap(): true });
    let text = std::fs::read_to_string(kept).unwrap();
    assert_eq!(
        serde_json::from_str::<serde_json::Value>(&text).unwrap(),
        expected
    );
    assert!(
        std::fs::read_dir(machine.clone_at("first"))
            .unwrap()
            .next()
            .is_none()
    );
}
