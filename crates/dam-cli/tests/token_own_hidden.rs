mod support;

use support::own::{listed, made_by, own_grant, own_hidden};
use support::tokens::{grant, grant_to};

const ASKS: [&[&str]; 3] = [
    &["show", "{}", "--json", "--no-pull"],
    &["edit", "{}", "--subject", "x", "--json"],
    &["done", "{}", "--json"],
];

fn asked(ask: &[&str], oid: &str) -> Vec<String> {
    ask.iter().map(|a| a.replace("{}", oid)).collect()
}

fn answers_alike_once_removed(h: &support::hidden::Hidden, mut make: impl FnMut() -> String) {
    for ask in ASKS {
        for prefix in [None, Some(7)] {
            let hidden = make();
            let named = prefix.map_or(hidden.as_str(), |n| &hidden[..n]);
            let line = asked(ask, named);
            let line: Vec<&str> = line.iter().map(String::as_str).collect();
            let out = h.answers_alike_once_removed(&hidden, &line);
            assert!(!out.status.success(), "{line:?}");
        }
    }
}

#[test]
fn the_owners_task_answers_an_own_token_like_a_missing_one() {
    let _guard = support::guard("the_owners_task_answers_an_own_token_like_a_missing_one")
        .allow_slow("six asks, each run hidden and again once removed");
    let h = own_hidden(own_grant());
    answers_alike_once_removed(&h, || h.task("diary", "work"));
}

#[test]
fn another_tokens_task_answers_an_own_token_like_a_missing_one() {
    let _guard = support::guard("another_tokens_task_answers_an_own_token_like_a_missing_one")
        .allow_slow("six asks, each run hidden and again once removed");
    let h = own_hidden(own_grant());
    let other = grant_to(&h.sb, "claude", 2, vec![grant(&["task:create"])]);
    let mut n = 0;
    answers_alike_once_removed(&h, || {
        n += 1;
        made_by(&h.sb, &other, &format!("chore {n}"), "work")
    });
}

#[test]
fn own_and_paths_both_hold_for_a_task_to_be_seen() {
    let _guard = support::guard("own_and_paths_both_hold_for_a_task_to_be_seen");
    let h = own_hidden(own_grant().paths(&["work"]));
    h.task("diary", "work");
    assert_eq!(listed(&h, &[]), vec![h.report.clone()]);
    let home = ["new", "laundry", "--path", "home", "--json"];
    assert_eq!(support::tokens::refused(&h.holding(&home)), "not_permitted");
    h.owner(&["mv", &h.report, "home/"]);
    assert_eq!(listed(&h, &[]), Vec::<String>::new());
    let edit = ["edit", &h.report, "--subject", "x", "--json"];
    let out = h.answers_alike_once_removed(&h.report, &edit);
    assert_eq!(out.status.code(), Some(4));
}
