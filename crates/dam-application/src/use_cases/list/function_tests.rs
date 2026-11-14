use std::path::PathBuf;

use dam_domain::Object;
use dam_domain::Permission::{HistoryRead, TaskRead};

use crate::config::{Config, FilterConfig, FilterRule};
use crate::errors::{FilterError, UseCaseError};
use crate::ports::{FilterFailure, FilterFault, History};
use crate::service::Dam;
use crate::testing::prelude::{CommitRepository, ObjectRepository};
use crate::testing::{FakeFilterFunctions, Scenario, allowed, holder, task_in};

fn filtered_by(rule: FilterRule, keeping: &[&'static str]) -> Scenario {
    let mut s = Scenario::default().configured(Config {
        filters: vec![FilterConfig {
            name: "slipping".into(),
            rule,
        }],
        ..Config::default()
    });
    s.fixture.filter_functions = FakeFilterFunctions::keeping(keeping);
    s.committed(Object::Task(task_in(1, "report", "work")));
    s.committed(Object::Task(task_in(2, "invoice", "work")));
    s.committed(Object::Task(task_in(3, "laundry", "home")));
    s
}

fn subjects(objects: &[Object]) -> Vec<&str> {
    objects.iter().map(|o| o.base().subject.as_str()).collect()
}

#[test]
fn a_function_filter_keeps_what_its_function_keeps() {
    let s = filtered_by(FilterRule::Function, &["laundry", "report"]);
    let listed = s.dam().list(Some("slipping")).unwrap();
    assert_eq!(subjects(&listed), ["laundry", "report"]);
}

#[test]
fn the_query_narrows_first_and_the_function_never_sees_what_it_dropped() {
    let s = filtered_by(
        FilterRule::QueryThenFunction("path:work/".into()),
        &["report", "laundry"],
    );
    let listed = s.dam().list(Some("slipping")).unwrap();
    assert_eq!(subjects(&listed), ["report"]);
    assert_eq!(
        s.fixture.filter_functions.seen_subjects(),
        ["invoice", "report"]
    );
}

#[test]
fn the_function_keeps_the_order_the_listing_gives() {
    let s = filtered_by(FilterRule::Function, &["report", "laundry", "invoice"]);
    let unfiltered = s.dam().list(None).unwrap();
    let listed = s.dam().list(Some("slipping")).unwrap();
    assert_eq!(subjects(&listed), subjects(&unfiltered));
}

#[test]
fn the_function_is_handed_each_objects_own_changes_newest_first() {
    let s = filtered_by(FilterRule::Function, &[]);
    s.dam().list(Some("slipping")).unwrap();
    let seen = s.fixture.filter_functions.seen.borrow();
    let (subject, Some(History::Recorded(commits))) = &seen[0] else {
        panic!("the owner reads the history");
    };
    assert_eq!(subject, "laundry");
    assert_eq!(commits.len(), 1);
    assert!(
        commits[0]
            .changes
            .iter()
            .all(|c| c.change.object().base().subject == "laundry")
    );
}

#[test]
fn a_token_has_the_function_run_only_over_objects_it_may_read() {
    let s = filtered_by(FilterRule::Function, &["report", "laundry"]);
    let token = holder(vec![allowed(&[TaskRead], Some(&["home"]))]);
    let dam = Dam {
        access: &token,
        ..s.dam()
    };
    assert_eq!(subjects(&dam.list(Some("slipping")).unwrap()), ["laundry"]);
    assert_eq!(s.fixture.filter_functions.seen_subjects(), ["laundry"]);
}

#[test]
fn a_token_without_history_read_has_the_history_withheld() {
    let s = filtered_by(FilterRule::Function, &[]);
    let reader = holder(vec![allowed(&[TaskRead], None)]);
    let historian = holder(vec![allowed(&[TaskRead, HistoryRead], None)]);
    for token in [&reader, &historian] {
        Dam {
            access: token,
            ..s.dam()
        }
        .list(Some("slipping"))
        .unwrap();
    }
    let seen = s.fixture.filter_functions.seen.borrow();
    assert_eq!(seen[0].1, Some(History::Withheld));
    assert!(matches!(seen[3].1, Some(History::Recorded(_))));
}

fn raised() -> FilterFailure {
    FilterFailure {
        file: PathBuf::from("/config/config.lua"),
        line: Some(4),
        fault: FilterFault::Raised {
            message: "boom".into(),
        },
    }
}

#[test]
fn a_failing_function_names_the_filter_and_its_place_to_the_owner() {
    let s = filtered_by(FilterRule::Function, &[]);
    *s.fixture.filter_functions.failing.borrow_mut() = Some(raised());
    assert_eq!(
        s.dam().list(Some("slipping")).unwrap_err(),
        UseCaseError::Filter(FilterError::Failed {
            filter: "slipping".into(),
            failure: raised(),
        })
    );
}

#[test]
fn a_failing_function_names_only_the_filter_to_a_token() {
    let s = filtered_by(FilterRule::Function, &[]);
    *s.fixture.filter_functions.failing.borrow_mut() = Some(raised());
    let token = holder(vec![allowed(&[TaskRead], None)]);
    let dam = Dam {
        access: &token,
        ..s.dam()
    };
    assert_eq!(
        dam.list(Some("slipping")).unwrap_err(),
        UseCaseError::Filter(FilterError::Withheld {
            filter: "slipping".into(),
        })
    );
}

#[test]
fn a_token_reads_no_details_of_a_change_another_sender_made() {
    let s = filtered_by(FilterRule::Function, &[]);
    let laundry = s.store.get(&crate::testing::oid(3)).unwrap().unwrap();
    s.store
        .commit(&dam_domain::CommitRecord {
            id: dam_domain::CommitId::generate(&mut |b: &mut [u8]| b.fill(50)),
            message: "noted".into(),
            at: dam_domain::Timestamp::UNIX_EPOCH,
            changes: vec![dam_domain::Authored {
                details: Some(dam_domain::Details {
                    meta: Default::default(),
                    host: Some("host".into()),
                    dir: None,
                    repo: None,
                    branch: None,
                    at: dam_domain::Timestamp::UNIX_EPOCH,
                    channel: dam_domain::Channel::Local,
                }),
                ..dam_domain::Authored::by_owner(dam_domain::Change::Create { after: laundry })
            }],
        })
        .unwrap();
    let everything = holder(vec![allowed(&[TaskRead, HistoryRead], None)]);
    s.dam().list(Some("slipping")).unwrap();
    Dam {
        access: &everything,
        ..s.dam()
    }
    .list(Some("slipping"))
    .unwrap();
    let seen = s.fixture.filter_functions.seen.borrow();
    let details_of = |at: usize| match &seen[at].1 {
        Some(History::Recorded(commits)) => commits[0].changes[0].details.is_some(),
        other => panic!("history:read on everything reads the history: {other:?}"),
    };
    assert!(details_of(0), "the owner reads every change's details");
    assert!(
        !details_of(3),
        "a token reads only its own changes' details"
    );
}
