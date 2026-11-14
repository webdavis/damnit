use std::cmp::Ordering;

use dam_domain::{Date, Object, QueryError, Timestamp, matches, parse_query};

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
        let (today, zone) = (self.clock.today(), self.clock.zone());
        let seen = self
            .store
            .all()?
            .into_iter()
            .filter(|o| self.access.sees(o))
            .map(|o| self.as_seen(o))
            .collect::<Result<Vec<Object>, _>>()?;
        let mut out: Vec<Object> = seen
            .into_iter()
            .filter(|o| expr.as_ref().is_none_or(|e| matches(e, o, today, &zone)))
            .collect();
        out.sort_by(|a, b| order(a, b, today));
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

fn focused_at(o: &Object, today: Date) -> Option<Timestamp> {
    o.as_task()?.focus_on(today).map(|f| f.set_at)
}

fn order(a: &Object, b: &Object, today: Date) -> Ordering {
    match (focused_at(a, today), focused_at(b, today)) {
        (Some(x), Some(y)) => x.cmp(&y),
        (Some(_), None) => Ordering::Less,
        (None, Some(_)) => Ordering::Greater,
        (None, None) => Ordering::Equal,
    }
    .then_with(|| a.base().path.cmp(&b.base().path))
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
mod access_tests;

#[cfg(test)]
mod tests;
