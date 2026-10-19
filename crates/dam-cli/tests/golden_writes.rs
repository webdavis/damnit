mod support;

use support::fixture::{self, CALL_MOM, OAT_MILK, OFFSITE, STANDUP, run, short};
use support::golden::assert_golden_output;

#[test]
fn new_prints_the_golden_document() {
    let args = [
        "--json",
        "new",
        "water plants",
        "--due",
        "2026-09-26",
        "-p",
        "3",
        "--label",
        "home",
        "--path",
        "home/",
        "--deadline",
        "2026-09-28",
        "--body",
        "the fern first",
    ];
    assert_golden_output(&fixture::committed(), &args, "new.json");
}

#[test]
fn add_prints_the_golden_document() {
    let sb = fixture::seeded();
    assert_golden_output(&sb, &["--json", "add", &short(OAT_MILK)], "add.json");
}

#[test]
fn commit_prints_the_golden_document() {
    let sb = fixture::seeded();
    run(&sb, &["add", "-A"]);
    let args = ["--json", "commit", "-m", "seed the week"];
    assert_golden_output(&sb, &args, "commit.json");
}

#[test]
fn rm_prints_the_golden_document() {
    let sb = fixture::committed();
    assert_golden_output(&sb, &["--json", "rm", &short(OFFSITE)], "rm.json");
}

#[test]
fn mv_prints_the_golden_document() {
    let sb = fixture::committed();
    assert_golden_output(&sb, &["--json", "mv", &short(STANDUP), "home/"], "mv.json");
}

#[test]
fn done_prints_the_golden_document() {
    let sb = fixture::committed();
    assert_golden_output(&sb, &["--json", "done", &short(OAT_MILK)], "done.json");
}

#[test]
fn reset_prints_the_golden_document() {
    let sb = fixture::seeded();
    run(&sb, &["add", "-A"]);
    assert_golden_output(&sb, &["--json", "reset", &short(CALL_MOM)], "reset.json");
}

#[test]
fn restore_prints_the_golden_document() {
    let sb = fixture::committed();
    run(
        &sb,
        &["edit", &short(OAT_MILK), "--subject", "buy soy milk"],
    );
    let args = ["--json", "restore", &short(OAT_MILK), &short(CALL_MOM)];
    assert_golden_output(&sb, &args, "restore.json");
}

#[test]
fn edit_prints_the_golden_document() {
    let sb = fixture::committed();
    let oat_milk = short(OAT_MILK);
    let args = [
        "--json",
        "edit",
        &oat_milk,
        "--subject",
        "buy soy milk",
        "--unlabel",
        "errand",
        "--label",
        "home",
        "--deadline",
        "2026-09-30",
        "--depends",
        &short(CALL_MOM),
    ];
    assert_golden_output(&sb, &args, "edit.json");
}
