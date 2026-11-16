use std::path::PathBuf;

use dam_domain::Object;

use crate::config::{Config, FilterConfig};
use crate::errors::UseCaseError;
use crate::ports::{FunctionFailure, FunctionFault};
use crate::testing::{FakeFilterFunctions, Scenario, oid, saved_function, task_in};
use crate::use_cases::list::trial::FilterTrial;

fn filtered_by(filter: FilterConfig, functions: FakeFilterFunctions) -> Scenario {
    let mut s = Scenario::default().configured(Config {
        filters: vec![filter],
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
    let s = filtered_by(saved_function("slipping", &[("path", "work/")]), functions);
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
    let s = filtered_by(saved_function("slipping", &[]), functions);
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
    let s = filtered_by(saved_function("slipping", &[]), functions);
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
fn a_table_that_does_not_read_is_the_selection_error() {
    let s = filtered_by(
        crate::testing::saved("slipping", &[("due", "soon")]),
        FakeFilterFunctions::default(),
    );
    assert!(matches!(
        s.dam().try_filter("slipping"),
        Err(UseCaseError::Select(_))
    ));
}
