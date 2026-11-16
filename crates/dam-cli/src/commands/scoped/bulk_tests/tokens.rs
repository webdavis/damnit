use dam_application::testing::{allowed, holder};
use dam_domain::Permission::{TaskChange, TaskComplete, TaskRead, TaskRemove};

use super::*;

fn token_over(paths: &[&str]) -> Context {
    let mut ctx = holding_a_project_and_more();
    ctx.access = Box::new(holder(vec![allowed(
        &[TaskRead, TaskChange, TaskComplete, TaskRemove],
        Some(paths),
    )]));
    ctx
}

#[test]
fn a_token_picks_only_what_it_sees_within_the_project() {
    for (paths, expected) in [
        (&["garden"][..], &[][..]),
        (&["damnit/cli", "home"][..], &["parse flags"][..]),
        (
            &["damnit", "cli", "home"][..],
            &["parse flags", "readme"][..],
        ),
    ] {
        for verb in [&["done"][..], &["edit", "--subject", "x"][..], &["rm"][..]] {
            let picked = picked(token_over(paths), verb, &["--query", "kind:task"], inside());
            assert_eq!(picked, expected, "dam {verb:?} over {paths:?}");
        }
    }
}
