use std::cmp::Ordering;

use dam_domain::{Date, Object, QueryError, matches, parse_query};

use crate::errors::{Refusal, UseCaseError};
use crate::service::Dam;

impl Dam<'_> {
    pub fn list(&self, query: Option<&str>) -> Result<Vec<Object>, UseCaseError> {
        let config = self.config;
        let expr = match query {
            None => None,
            Some(q) => {
                let text = config.filter(q).map(|f| f.query.as_str()).unwrap_or(q);
                Some(parse_query(text, &config.categories).map_err(query_error)?)
            }
        };
        let today = self.clock.today();
        let mut out: Vec<Object> = self
            .store
            .all()?
            .into_iter()
            .filter(|o| expr.as_ref().is_none_or(|e| matches(e, o, today)))
            .collect();
        out.sort_by(order);
        Ok(out)
    }
}

fn query_error(e: QueryError) -> UseCaseError {
    match e {
        QueryError::UnknownCategory(name) => Refusal::UnknownCategory(name).into(),
        QueryError::NotAValue { category, value } => {
            Refusal::NotACategoryValue { category, value }.into()
        }
        e => UseCaseError::Query(e),
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
    use crate::config::{Config, FilterConfig};
    use crate::testing::{Scenario, oid};
    use dam_domain::{Categories, Category, Object, Path, Priority, Task, When};
    use jiff::civil::date;

    fn tasks_a_b_and_c_later(config: Config) -> Scenario {
        let s = Scenario::default().configured(config);
        let mut a = Task::new(oid(1), "a");
        a.due = Some(When::Day(date(2026, 9, 18)));
        a.base.labels.insert("deep".into());
        let mut b = Task::new(oid(2), "b");
        b.due = Some(When::Day(date(2026, 9, 10)));
        b.priority = Priority::HIGHEST;
        let mut c = Task::new(oid(3), "c");
        c.base.path = Path::parse("later").unwrap();
        for t in [a, b, c] {
            s.working(Object::Task(t));
        }
        s
    }

    fn effort_categories_and_a_today_filter() -> Config {
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
    fn no_query_lists_everything_by_path_then_earlier_due_first() {
        let s = tasks_a_b_and_c_later(effort_categories_and_a_today_filter());
        let out = s.dam().list(None).unwrap();
        let subjects: Vec<&str> = out.iter().map(|o| o.base().subject.as_str()).collect();
        assert_eq!(subjects, vec!["b", "a", "c"]);
    }

    #[test]
    fn a_saved_filter_runs_by_name() {
        let s = tasks_a_b_and_c_later(effort_categories_and_a_today_filter());
        let out = s.dam().list(Some("today")).unwrap();
        assert_eq!(out.len(), 2);
    }

    #[test]
    fn a_category_term_refuses_a_value_of_another_category() {
        let s = tasks_a_b_and_c_later(effort_categories_and_a_today_filter());
        let err = s.dam().list(Some("effort:home")).unwrap_err();
        assert_eq!(
            err,
            UseCaseError::Refused(Refusal::NotACategoryValue {
                category: "effort".into(),
                value: "home".into()
            })
        );
    }

    #[test]
    fn a_category_term_checks_the_category_exists() {
        let s = tasks_a_b_and_c_later(effort_categories_and_a_today_filter());
        assert_eq!(s.dam().list(Some("effort:deep")).unwrap().len(), 1);
        let err = s.dam().list(Some("mood:happy")).unwrap_err();
        assert_eq!(
            err,
            UseCaseError::Refused(Refusal::UnknownCategory("mood".into()))
        );
    }

    #[test]
    fn a_bad_query_is_a_query_error_naming_what_was_typed() {
        let s = Scenario::default().configured(effort_categories_and_a_today_filter());
        let err = s.dam().list(Some("(due:today")).unwrap_err();
        assert_eq!(err, UseCaseError::Query(QueryError::Unclosed));
    }
}
