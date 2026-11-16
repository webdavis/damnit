use dam_domain::{FilterTable, Path, TableValue};

use crate::args::selection::SelectArgs;

fn typed(path: &str, any: &str) -> SelectArgs {
    let one = |text: &str| TableValue::One(text.into());
    SelectArgs {
        typed: FilterTable {
            fields: [("path".into(), one(path))].into(),
            any: vec![FilterTable {
                fields: [("path".into(), one(any))].into(),
                any: Vec::new(),
            }],
        },
    }
}

fn paths(args: &SelectArgs) -> Vec<String> {
    let top = args.typed.fields["path"].texts().to_vec();
    top.into_iter()
        .chain(args.typed.any[0].fields["path"].texts().to_vec())
        .collect()
}

#[test]
fn a_typed_path_starts_from_the_place() {
    let place = Path::parse("damnit").unwrap();
    let args = typed("cli/,docs", "not:cli/").under(&place);
    assert_eq!(
        paths(&args),
        ["damnit/cli/,damnit/docs/", "not:damnit/cli/"]
    );
    let exact = typed("=a,b/", "/").under(&place);
    assert_eq!(paths(&exact), ["=damnit/a,b/", "damnit/"]);
}

#[test]
fn at_the_root_and_on_a_path_that_cannot_be_read_nothing_changes() {
    let args = typed("cli/", "/").under(&Path::root());
    assert_eq!(paths(&args), ["cli/", "/"]);
    let bad = typed("a//b", "x").under(&Path::parse("damnit").unwrap());
    assert_eq!(paths(&bad)[0], "a//b");
}
