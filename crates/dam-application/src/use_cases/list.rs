use std::cmp::Ordering;

use dam_domain::{Categories, Date, Expr, Object, Term, matches, parse_query};

use crate::config::Config;
use crate::errors::{Refusal, UseCaseError};
use crate::ports::{Clock, ObjectStore};

pub fn list(
    store: &dyn ObjectStore,
    clock: &dyn Clock,
    config: &Config,
    query: Option<&str>,
) -> Result<Vec<Object>, UseCaseError> {
    let expr = match query {
        None => None,
        Some(q) => {
            let text = config.filter(q).map(|f| f.query.as_str()).unwrap_or(q);
            let expr = parse_query(text).map_err(|e| UseCaseError::Parse(e.to_string()))?;
            check_categories(&expr, &config.categories)?;
            Some(expr)
        }
    };
    let today = clock.today();
    let mut out: Vec<Object> = store
        .all()?
        .into_iter()
        .filter(|o| expr.as_ref().is_none_or(|e| matches(e, o, today)))
        .collect();
    out.sort_by(order);
    Ok(out)
}

fn check_categories(expr: &Expr, categories: &Categories) -> Result<(), UseCaseError> {
    match expr {
        Expr::And(a, b) | Expr::Or(a, b) => {
            check_categories(a, categories)?;
            check_categories(b, categories)
        }
        Expr::Not(a) => check_categories(a, categories),
        Expr::Term(Term::Category { name, .. }) => {
            if categories.category_of_name(name).is_none() {
                return Err(Refusal::UnknownCategory(name.clone()).into());
            }
            Ok(())
        }
        Expr::Term(_) => Ok(()),
    }
}

fn date_of(o: &Object) -> Option<Date> {
    match o {
        Object::Task(t) => t.due.as_ref().map(|d| d.date()),
        Object::Event(e) => Some(e.start.date()),
    }
}

fn order(a: &Object, b: &Object) -> Ordering {
    a.base()
        .path
        .cmp(&b.base().path)
        .then_with(|| match (date_of(a), date_of(b)) {
            (Some(x), Some(y)) => x.cmp(&y),
            (Some(_), None) => Ordering::Less,
            (None, Some(_)) => Ordering::Greater,
            (None, None) => Ordering::Equal,
        })
        .then_with(|| {
            let pa = a.as_task().map(|t| t.priority.get()).unwrap_or(u8::MAX);
            let pb = b.as_task().map(|t| t.priority.get()).unwrap_or(u8::MAX);
            pa.cmp(&pb)
        })
        .then_with(|| a.base().subject.cmp(&b.base().subject))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::FilterConfig;
    use crate::testing::{FixedClock, MemoryStore, oid};
    use dam_domain::{Categories, Category, Object, Path, Priority, Task, When};
    use jiff::civil::date;

    fn seed(store: &MemoryStore) {
        let mut a = Task::new(oid(1), "a");
        a.due = Some(When::Day(date(2026, 9, 18)));
        a.base.labels.insert("deep".into());
        let mut b = Task::new(oid(2), "b");
        b.due = Some(When::Day(date(2026, 9, 10)));
        b.priority = Priority::HIGHEST;
        let mut c = Task::new(oid(3), "c");
        c.base.path = Path::parse("later").unwrap();
        for t in [a, b, c] {
            store.put(&Object::Task(t)).unwrap();
        }
    }

    fn config() -> Config {
        Config {
            categories: Categories::new(vec![Category {
                name: "effort".into(),
                values: vec!["deep".into()],
                exclusive: true,
            }])
            .unwrap(),
            filters: vec![FilterConfig {
                name: "today".into(),
                query: "due:today | overdue".into(),
            }],
            ..Config::default()
        }
    }

    #[test]
    fn no_query_lists_everything_ordered() {
        let store = MemoryStore::new();
        seed(&store);
        let out = list(&store, &FixedClock(date(2026, 9, 18)), &config(), None).unwrap();
        let subjects: Vec<&str> = out.iter().map(|o| o.base().subject.as_str()).collect();
        // root path first (b due 9/10 before a due 9/18), then later/
        assert_eq!(subjects, vec!["b", "a", "c"]);
    }

    #[test]
    fn a_saved_filter_runs_by_name() {
        let store = MemoryStore::new();
        seed(&store);
        let out = list(
            &store,
            &FixedClock(date(2026, 9, 18)),
            &config(),
            Some("today"),
        )
        .unwrap();
        assert_eq!(out.len(), 2);
    }

    #[test]
    fn a_category_term_checks_the_category_exists() {
        let store = MemoryStore::new();
        seed(&store);
        assert_eq!(
            list(
                &store,
                &FixedClock(date(2026, 9, 18)),
                &config(),
                Some("effort:deep")
            )
            .unwrap()
            .len(),
            1
        );
        let err = list(
            &store,
            &FixedClock(date(2026, 9, 18)),
            &config(),
            Some("mood:happy"),
        )
        .unwrap_err();
        assert_eq!(
            err,
            UseCaseError::Refused(Refusal::UnknownCategory("mood".into()))
        );
    }

    #[test]
    fn a_bad_query_is_a_parse_error() {
        let store = MemoryStore::new();
        let err = list(
            &store,
            &FixedClock(date(2026, 9, 18)),
            &config(),
            Some("(due:today"),
        )
        .unwrap_err();
        assert!(matches!(err, UseCaseError::Parse(_)));
    }
}
