use crate::support::sandbox::Sandbox;

#[test]
fn a_test_run_reaches_only_the_sandboxs_own_scheduler_programs() {
    let sb = Sandbox::new();
    let own = sb.dir.path().join("bin");
    for program in ["launchctl", "systemctl"] {
        let out = sb.shell(&format!("command -v {program}")).output().unwrap();
        let found = String::from_utf8_lossy(&out.stdout);
        assert_eq!(found.trim(), own.join(program).to_str().unwrap());
    }
}
