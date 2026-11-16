use std::path::PathBuf;

use dam_application::testing::{FakeFilterFunctions, FakeProgress, Said, Scenario, task_in};
use dam_application::{Config, FilterConfig, FilterRule, FunctionFailure, FunctionFault};
use dam_domain::Object;

use super::{attention_of, json, rows_of};
use crate::commands::check_report::CheckReport;
use crate::commands::test::filters::tried;

fn saved(filters: &[(&str, FilterRule)], functions: FakeFilterFunctions) -> Scenario {
    let filters = filters
        .iter()
        .map(|(name, rule)| FilterConfig {
            name: name.to_string(),
            rule: rule.clone(),
        })
        .collect();
    let mut s = Scenario::default().configured(Config {
        filters,
        ..Config::default()
    });
    s.fixture.filter_functions = functions;
    s.committed(Object::Task(task_in(1, "report", "work")));
    s.committed(Object::Task(task_in(2, "invoice", "work")));
    s
}

fn report_of(s: &Scenario, progress: &FakeProgress) -> CheckReport {
    CheckReport::new("test", super::super::PARTS, vec![tried(&s.dam(), progress)])
}

fn nil_due() -> FunctionFailure {
    FunctionFailure {
        file: PathBuf::from("/home/someone/.config/dam/config.lua"),
        line: Some(4),
        fault: FunctionFault::Raised {
            message: "attempt to index a nil value (field 'due')".into(),
        },
    }
}

const BROKEN: &str = "broken: failed on 0202020 at /home/someone/.config/dam/config.lua:4: \
                      attempt to index a nil value (field 'due')";

#[test]
fn a_filter_that_runs_passes_with_how_many_it_ran_on_and_kept() {
    let s = saved(
        &[("slipping", FilterRule::Function)],
        FakeFilterFunctions::keeping(&["report"]),
    );
    let progress = FakeProgress::default();
    let report = report_of(&s, &progress);
    assert_eq!(
        rows_of(&report),
        [
            "  ✓ filters: 1 filter: 1 ok",
            "    ✓ slipping: ran on 2 objects, kept 1"
        ]
    );
    assert_eq!(
        *progress.said.borrow(),
        [
            Said::Started("try_filter: trying filter slipping ".into(), None),
            Said::Finished
        ]
    );
}

#[test]
fn a_function_that_errors_on_one_object_fails_naming_the_object_and_the_line() {
    let functions = FakeFilterFunctions {
        failing_on: Some(("invoice", nil_due())),
        ..FakeFilterFunctions::default()
    };
    let s = saved(&[("broken", FilterRule::Function)], functions);
    let progress = FakeProgress::default();
    let report = report_of(&s, &progress);
    assert_eq!(
        rows_of(&report),
        [
            "  ✗ filters: 1 filter: 1 issue".to_string(),
            format!("    ✗ {BROKEN}"),
            "      → fix filter.broken in config.lua, then run dam test filters".to_string(),
        ]
    );
    assert_eq!(progress.said.borrow()[1], Said::Failed);
}

#[test]
fn a_function_that_runs_too_long_fails() {
    let functions = FakeFilterFunctions::default();
    *functions.failing.borrow_mut() = Some(FunctionFailure {
        fault: FunctionFault::RanOver,
        ..nil_due()
    });
    let s = saved(&[("slow", FilterRule::Function)], functions);
    let report = report_of(&s, &FakeProgress::default());
    assert_eq!(
        rows_of(&report)[1],
        "    ✗ slow: failed at /home/someone/.config/dam/config.lua:4: \
         match ran past the time limit for one listing"
    );
}

#[test]
fn a_query_that_does_not_parse_fails_with_the_query_error() {
    let s = saved(
        &[("odd", FilterRule::Query("due:(".into()))],
        FakeFilterFunctions::default(),
    );
    let report = report_of(&s, &FakeProgress::default());
    assert!(
        rows_of(&report)[1].starts_with("    ✗ odd: the query does not parse: "),
        "{:?}",
        rows_of(&report)
    );
}

#[test]
fn no_saved_filters_is_one_passing_line() {
    let s = saved(&[], FakeFilterFunctions::default());
    let report = report_of(&s, &FakeProgress::default());
    assert_eq!(rows_of(&report), ["  ✓ filters: no saved filters"]);
}

#[test]
fn several_filters_put_the_worst_on_the_part_and_in_the_json() {
    let functions = FakeFilterFunctions {
        failing_on: Some(("invoice", nil_due())),
        ..FakeFilterFunctions::keeping(&["report"])
    };
    let s = saved(
        &[
            ("work", FilterRule::Query("path:work/".into())),
            ("broken", FilterRule::QueryThenFunction("path:work/".into())),
        ],
        functions,
    );
    let report = report_of(&s, &FakeProgress::default());
    assert_eq!(rows_of(&report)[0], "  ✗ filters: 2 filters: 1 issue, 1 ok");
    assert_eq!(
        attention_of(&report),
        [
            "  ✗ 1 issue to fix".to_string(),
            format!("  1. filters for {BROKEN}")
        ]
    );
    let document = json(&report);
    let part = &document["checks"][0];
    assert_eq!(part["name"], "filters");
    assert_eq!(part["result"], "issue");
    assert_eq!(part["rows"][0]["name"], "work");
    assert_eq!(part["rows"][0]["detail"], "ran on 2 objects, kept 2");
    assert_eq!(part["rows"][1]["result"], "issue");
    super::report::assert_matches_the_published_schema(&document);
}
