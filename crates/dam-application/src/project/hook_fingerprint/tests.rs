use std::collections::BTreeMap;

use super::{HookFileChange, HookFingerprint};

fn of(files: &[(&str, &str)]) -> HookFingerprint {
    HookFingerprint::of(files.iter().map(|(name, text)| (*name, text.as_bytes())))
}

#[test]
fn the_same_files_give_the_same_fingerprint_in_any_order() {
    let one = of(&[("a.lua", "x"), ("b.lua", "y")]);
    let other = of(&[("b.lua", "y"), ("a.lua", "x")]);
    assert_eq!(one, other);
    assert!(one.changes(&other).is_empty());
}

#[test]
fn a_file_is_kept_by_the_sha256_of_its_bytes() {
    let kept = of(&[("a.lua", "abc")]);
    assert_eq!(
        kept.files(),
        &BTreeMap::from([(
            "a.lua".to_string(),
            "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad".to_string()
        )])
    );
}

#[test]
fn changed_new_and_removed_files_are_each_named() {
    let then = of(&[("kept.lua", "x"), ("edited.lua", "old"), ("gone.lua", "z")]);
    let now = of(&[("kept.lua", "x"), ("edited.lua", "new"), ("added.lua", "w")]);
    assert_ne!(then, now);
    assert_eq!(
        then.changes(&now),
        BTreeMap::from([
            ("added.lua".to_string(), HookFileChange::New),
            ("edited.lua".to_string(), HookFileChange::Changed),
            ("gone.lua".to_string(), HookFileChange::Removed),
        ])
    );
}

#[test]
fn a_renamed_file_is_a_change() {
    let then = of(&[("task_new.lua", "x")]);
    let now = of(&[("task_done.lua", "x")]);
    assert_ne!(then, now);
}

#[test]
fn a_stored_fingerprint_reads_back_equal() {
    let kept = of(&[("a.lua", "abc")]);
    assert_eq!(HookFingerprint::from_files(kept.files().clone()), kept);
}

#[test]
fn a_link_a_folder_and_a_file_of_the_same_bytes_differ() {
    use super::HookEntry;
    let as_file = HookFingerprint::of_tree([("x".to_string(), HookEntry::File(b"t"))]);
    let as_link = HookFingerprint::of_tree([("x".to_string(), HookEntry::Link(b"t"))]);
    let as_folder = HookFingerprint::of_tree([("x".to_string(), HookEntry::Folder)]);
    assert_ne!(as_file, as_link);
    assert_ne!(as_file, as_folder);
    assert_ne!(as_link, as_folder);
    assert_eq!(as_file, of(&[("x", "t")]));
}
