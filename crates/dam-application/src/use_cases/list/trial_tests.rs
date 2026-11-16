use std::path::PathBuf;

use dam_domain::Object;

use crate::config::{Config, FilterConfig, FilterRule};
use crate::errors::UseCaseError;
use crate::ports::{FunctionFailure, FunctionFault};
use crate::testing::{FakeFilterFunctions, Scenario, oid, task_in};
use crate::use_cases::list::trial::FilterTrial;

fn filtered_by(rule: FilterRule, functions: FakeFilterFunctions) -> Scenario {
    let mut s = Scenario::default().configured(Config {
        filters: vec![FilterConfig {
            name: "slipping".into(),
            rule,
        }],
        ..Config::default()
    });
    s.fixture.filter_functions = functions;
    s.committed(Object::Task(task_in(1, "report", "work")));
    s.committed(Object::Task(task_in(2, "invoice", "work")));
    s.committed(Object::Task(task_in(3, "laundry", "home")));
    s
}

fn raised() -> FunctionFailure {
    FunctionFailure {
        file: PathBuf::from("config.lua"),
        line: Some(4),
        fault: FunctionFault::Raised {
            message: "attempt to index a nil value (field 'due')".into(),
        },
    }
}

#[test]
fn a_filter_that_runs_reports_how_many_objects_it_ran_on_and_kept() {
    let functions = FakeFilterFunctions::keeping(&["report"]);
    let s = filtered_by(
        FilterRule::QueryThenFunction("path:work/".into()),
        functions,
    );
    assert_eq!(
        s.dam().try_filter("slipping").unwrap(),
        FilterTrial::Passed { ran_on: 3, kept: 1 }
    );
}

#[test]
fn a_function_that_fails_on_one_object_names_that_object() {
    let functions = FakeFilterFunctions {
        failing_on: Some(("invoice", raised())),
        ..FakeFilterFunctions::default()
    };
    let s = filtered_by(FilterRule::Function, functions);
    assert_eq!(
        s.dam().try_filter("slipping").unwrap(),
        FilterTrial::Failed {
            failure: raised(),
            object: Some(oid(2)),
        }
    );
}

#[test]
fn a_function_that_runs_over_names_no_object_and_is_never_run_again() {
    let ran_over = FunctionFailure {
        fault: FunctionFault::RanOver,
        ..raised()
    };
    let functions = FakeFilterFunctions::default();
    *functions.failing.borrow_mut() = Some(ran_over.clone());
    let s = filtered_by(FilterRule::Function, functions);
    assert_eq!(
        s.dam().try_filter("slipping").unwrap(),
        FilterTrial::Failed {
            failure: ran_over,
            object: None,
        }
    );
    assert_eq!(s.fixture.filter_functions.seen_subjects().len(), 3);
}

#[test]
fn a_query_that_does_not_parse_is_the_query_error() {
    let s = filtered_by(
        FilterRule::Query("due:(".into()),
        FakeFilterFunctions::default(),
    );
    assert!(matches!(
        s.dam().try_filter("slipping"),
        Err(UseCaseError::Query(_))
    ));
}
