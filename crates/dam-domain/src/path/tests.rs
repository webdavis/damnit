use super::*;

#[test]
fn root_is_empty_and_has_no_parent() {
    assert_eq!(Path::root().as_str(), "");
    assert_eq!(Path::root().parent(), None);
    assert_eq!(Path::root().depth(), 0);
}

#[test]
fn default_is_the_root() {
    assert_eq!(Path::default(), Path::root());
}

#[test]
fn parse_normalizes_a_trailing_slash() {
    assert_eq!(Path::parse("a/b").unwrap().as_str(), "a/b/");
    assert_eq!(Path::parse("a/b/").unwrap().as_str(), "a/b/");
}

#[test]
fn parse_refuses_an_empty_segment() {
    assert_eq!(Path::parse("a//b"), Err(PathError::EmptySegment));
}

#[test]
fn parse_refuses_a_relative_segment_so_a_dot_dot_b_cannot_pass_as_inside_a() {
    for text in ["a/../b", "a/./b", "..", ".", "../a", "./a", "a/..", "a/."] {
        assert_eq!(
            Path::parse(text),
            Err(PathError::RelativeSegment),
            "{text:?} parsed"
        );
    }
}

#[test]
fn join_refuses_a_relative_segment() {
    let a = Path::parse("a").unwrap();
    assert_eq!(a.join(".."), Err(PathError::RelativeSegment));
    assert_eq!(a.join("."), Err(PathError::RelativeSegment));
}

#[test]
fn parent_drops_the_last_segment() {
    let p = Path::parse("a/b/c").unwrap();
    assert_eq!(p.parent().unwrap().as_str(), "a/b/");
    assert_eq!(Path::parse("a").unwrap().parent(), Some(Path::root()));
}

#[test]
fn is_within_is_true_for_a_strict_descendant_and_for_self() {
    let a = Path::parse("a").unwrap();
    let ab = Path::parse("a/b").unwrap();
    assert!(ab.is_within(&a));
    assert!(a.is_within(&a));
    assert!(!a.is_within(&ab));
    assert!(ab.is_within(&Path::root()));
}

#[test]
fn join_appends_one_segment() {
    assert_eq!(Path::root().join("x").unwrap().as_str(), "x/");
    assert_eq!(
        Path::parse("a").unwrap().join("b").unwrap().as_str(),
        "a/b/"
    );
    assert_eq!(Path::root().join("a/b"), Err(PathError::BadCharacter('/')));
}

#[test]
fn rebase_moves_a_path_from_one_folder_to_another() {
    let path = Path::parse("work/plan/step").unwrap();
    let rebased = path.rebase(
        &Path::parse("work/plan").unwrap(),
        &Path::parse("home").unwrap(),
    );
    assert_eq!(rebased.unwrap().as_str(), "home/step/");
}

#[test]
fn rebase_into_or_out_of_the_root() {
    let root = Path::root();
    let ab = Path::parse("a/b").unwrap();
    assert_eq!(
        ab.rebase(&root, &Path::parse("x").unwrap())
            .unwrap()
            .as_str(),
        "x/a/b/"
    );
    assert_eq!(
        ab.rebase(&Path::parse("a").unwrap(), &root)
            .unwrap()
            .as_str(),
        "b/"
    );
    assert_eq!(root.rebase(&root, &root), Some(Path::root()));
}

#[test]
fn rebase_keeps_a_multibyte_segment_whole() {
    let path = Path::parse("été/über/ß").unwrap();
    let rebased = path.rebase(&Path::parse("été").unwrap(), &Path::parse("日本").unwrap());
    assert_eq!(rebased.unwrap().as_str(), "日本/über/ß/");
}

#[test]
fn rebase_is_none_outside_the_folder_even_when_a_name_shares_its_prefix() {
    let path = Path::parse("ab/c").unwrap();
    assert_eq!(path.rebase(&Path::parse("a").unwrap(), &Path::root()), None);
}
