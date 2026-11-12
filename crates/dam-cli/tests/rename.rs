mod support;

use std::os::unix::fs::PermissionsExt;

use support::sandbox::Sandbox;

fn path_of(sb: &Sandbox, oid: &str) -> String {
    let (ok, out, err) = sb.dam(&["show", &oid[..7], "--json"]);
    assert!(ok, "{err}");
    let json: serde_json::Value = serde_json::from_str(&out).unwrap();
    json["path"].as_str().unwrap().to_string()
}

fn parent_with_family(sb: &Sandbox) -> (String, String, String) {
    let parent = sb.new_object(&["parent"]);
    let child = sb.new_object(&["child", "--path", "parent/"]);
    let grandchild = sb.new_object(&["grandchild", "--path", "parent/child/"]);
    (parent, child, grandchild)
}

#[test]
fn renaming_a_subject_with_the_flag_carries_its_children() {
    let _guard = support::guard("renaming_a_subject_with_the_flag_carries_its_children");
    let sb = Sandbox::new();
    let (parent, child, grandchild) = parent_with_family(&sb);

    let (ok, _, err) = sb.dam(&["edit", &parent[..7], "--subject", "renamed"]);
    assert!(ok, "{err}");

    assert_eq!(path_of(&sb, &child), "renamed/");
    assert_eq!(path_of(&sb, &grandchild), "renamed/child/");
}

#[test]
fn renaming_a_subject_in_the_editor_carries_its_children() {
    let _guard = support::guard("renaming_a_subject_in_the_editor_carries_its_children");
    let sb = Sandbox::new();
    let (parent, child, grandchild) = parent_with_family(&sb);
    let renames = sb.dir.path().join("bin/rename-subject");
    std::fs::write(
        &renames,
        "#!/bin/sh\nsed 's/^subject = .*/subject = \"renamed\"/' \"$1\" > \"$1.new\" && mv \"$1.new\" \"$1\"\n",
    )
    .unwrap();
    std::fs::set_permissions(&renames, std::fs::Permissions::from_mode(0o755)).unwrap();

    let out = sb
        .command(&["edit", "-e", &parent[..7]])
        .env("EDITOR", &renames)
        .env("VISUAL", &renames)
        .output()
        .unwrap();
    assert!(
        out.status.success(),
        "{}",
        String::from_utf8_lossy(&out.stderr)
    );

    assert_eq!(path_of(&sb, &child), "renamed/");
    assert_eq!(path_of(&sb, &grandchild), "renamed/child/");
}

#[test]
fn a_subject_that_cannot_name_a_folder_is_refused_while_the_folder_holds_objects() {
    let _guard = support::guard(
        "a_subject_that_cannot_name_a_folder_is_refused_while_the_folder_holds_objects",
    );
    let sb = Sandbox::new();
    let (parent, child, _) = parent_with_family(&sb);

    for subject in [".", "..", ""] {
        let refused = sb.output(&["edit", &parent[..7], "--subject", subject]);
        assert_eq!(refused.status.code(), Some(4));
        assert_eq!(
            String::from_utf8_lossy(&refused.stderr),
            format!(
                "dam: {} has objects in its folder, so its subject cannot be empty, . or ..\n",
                &parent[..7]
            )
        );
    }
    let refused = sb.output(&["edit", &parent[..7], "--subject", ".", "--json"]);
    let document: serde_json::Value = serde_json::from_slice(&refused.stderr).unwrap();
    assert_eq!(document["error"]["rule"], "subject_names_no_folder");

    let (ok, out, err) = sb.dam(&["show", &parent[..7], "--json"]);
    assert!(ok, "{err}");
    let shown: serde_json::Value = serde_json::from_str(&out).unwrap();
    assert_eq!(shown["subject"], "parent");
    assert_eq!(path_of(&sb, &child), "parent/");
}

#[test]
fn an_object_with_an_empty_folder_may_take_a_subject_that_cannot_name_one() {
    let _guard =
        support::guard("an_object_with_an_empty_folder_may_take_a_subject_that_cannot_name_one");
    let sb = Sandbox::new();
    let lonely = sb.new_object(&["lonely"]);

    let (ok, _, err) = sb.dam(&["edit", &lonely[..7], "--subject", "."]);
    assert!(ok, "{err}");
}
