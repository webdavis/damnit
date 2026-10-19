mod support;

use support::sandbox::Sandbox;

fn ls(sb: &Sandbox) -> String {
    let (ok, out, err) = sb.dam(&["ls"]);
    assert!(ok, "{err}");
    out
}

#[test]
fn mv_of_a_top_level_object_moves_that_object_alone_and_back() {
    let _guard = support::guard("mv_of_a_top_level_object_moves_that_object_alone_and_back");
    let sb = Sandbox::new();
    let milk = sb.new_object(&["milk"]);
    let eggs = sb.new_object(&["eggs"]);
    let plan = sb.new_object(&["plan", "--path", "work"]);
    let note = sb.new_object(&["note", "--path", "home"]);
    let short = |oid: &str| oid[..7].to_string();
    let at_top = format!(
        "{}  p4  -  -      eggs\n{}  p4  -  -      milk\n{}  p4  -  home/  note\n{}  p4  -  work/  plan\n",
        short(&eggs),
        short(&milk),
        short(&note),
        short(&plan)
    );
    assert_eq!(ls(&sb), at_top);

    let (ok, out, err) = sb.dam(&["mv", &milk, "work/"]);
    assert!(ok, "{err}");
    assert_eq!(out, format!("{}  p4  -  work/  milk\n", short(&milk)));
    assert_eq!(
        ls(&sb),
        format!(
            "{}  p4  -  -      eggs\n{}  p4  -  home/  note\n{}  p4  -  work/  milk\n{}  p4  -  work/  plan\n",
            short(&eggs),
            short(&note),
            short(&milk),
            short(&plan)
        )
    );

    assert!(sb.dam(&["mv", &milk, ""]).0);
    assert_eq!(ls(&sb), at_top);

    assert!(sb.dam(&["mv", &milk, "work/"]).0);
    assert!(sb.dam(&["mv", &milk, "/"]).0);
    assert_eq!(ls(&sb), at_top);
}
