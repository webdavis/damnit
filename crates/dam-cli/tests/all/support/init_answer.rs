use super::sandbox::Sandbox;

pub fn init_in_a_folder_of_its_own() -> serde_json::Value {
    let sb = Sandbox::new();
    let project = sb.dir.path().join("app");
    std::fs::create_dir_all(&project).unwrap();
    let out = sb
        .command(&["init", "--json"])
        .current_dir(project)
        .output()
        .unwrap();
    assert!(out.status.success(), "{out:?}");
    serde_json::from_slice(&out.stdout).unwrap()
}
