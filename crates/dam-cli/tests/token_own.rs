mod support;

use support::own::{listed, own_grant, own_hidden, send};

#[test]
fn a_token_reads_edits_and_completes_the_task_it_made() {
    let _guard = support::guard("a_token_reads_edits_and_completes_the_task_it_made");
    let h = own_hidden(own_grant());
    let shown = h.holding(&["show", &h.report, "--json", "--no-pull"]);
    assert!(shown.status.success(), "{shown:?}");
    let edit = send(
        &h.sb,
        &h.held,
        &["edit", &h.report, "--subject", "final report"],
    );
    assert_eq!(edit["answer"], "changed");
    assert_eq!(edit["oid"], h.report.as_str());
    let done = h.holding(&["done", &h.report, "--json"]);
    assert!(done.status.success(), "{done:?}");
}

#[test]
fn a_tokens_ls_lists_only_what_it_made() {
    let _guard = support::guard("a_tokens_ls_lists_only_what_it_made");
    let h = own_hidden(own_grant());
    h.task("diary", "work");
    assert_eq!(listed(&h, &[]), vec![h.report.clone()]);
    assert_eq!(listed(&h, &["--by", "all"]), vec![h.report.clone()]);
    assert_eq!(listed(&h, &["--by", "mine"]), Vec::<String>::new());
}

#[test]
fn a_repeat_of_new_with_an_id_changes_the_task_the_token_made() {
    let _guard = support::guard("a_repeat_of_new_with_an_id_changes_the_task_the_token_made");
    let h = own_hidden(own_grant());
    let first = send(&h.sb, &h.held, &["new", "Call dentist", "--id", "dentist"]);
    let waiting = send(
        &h.sb,
        &h.held,
        &["new", "Call the dentist", "--id", "dentist"],
    );
    assert_eq!(waiting["answer"], "changed");
    assert_eq!(waiting["oid"], first["oid"]);
    support::own::adopt(&h.sb, first["hash"].as_str().unwrap());
    let adopted = send(&h.sb, &h.held, &["new", "Ring dentist", "--id", "dentist"]);
    assert_eq!(adopted["answer"], "changed");
    assert_eq!(adopted["oid"], first["oid"]);
}

#[test]
fn a_change_to_its_task_after_the_owners_edit_still_waits_for_the_owner() {
    let _guard =
        support::guard("a_change_to_its_task_after_the_owners_edit_still_waits_for_the_owner");
    let h = own_hidden(own_grant());
    h.owner(&["edit", &h.report, "--subject", "quarterly report"]);
    let edit = send(
        &h.sb,
        &h.held,
        &["edit", &h.report, "--subject", "Q3 report"],
    );
    assert_eq!(edit["answer"], "changed");
}
