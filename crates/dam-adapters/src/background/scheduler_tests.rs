use super::{Scheduler, scheduler_on};

#[test]
fn macos_uses_launchd() {
    let runtime = tempfile::tempdir().unwrap();
    std::fs::create_dir(runtime.path().join("systemd")).unwrap();
    let chosen = scheduler_on(true, Some(runtime.path().to_path_buf()));
    assert_eq!(chosen, Scheduler::Launchd);
}

#[test]
fn linux_uses_systemd_when_the_runtime_folder_holds_a_user_manager() {
    let runtime = tempfile::tempdir().unwrap();
    std::fs::create_dir(runtime.path().join("systemd")).unwrap();
    let chosen = scheduler_on(false, Some(runtime.path().to_path_buf()));
    assert_eq!(chosen, Scheduler::Systemd(runtime.path().to_path_buf()));
}

#[test]
fn linux_without_a_user_manager_has_no_scheduler() {
    let runtime = tempfile::tempdir().unwrap();
    let chosen = scheduler_on(false, Some(runtime.path().to_path_buf()));
    assert_eq!(chosen, Scheduler::None);
    assert_eq!(scheduler_on(false, None), Scheduler::None);
}
