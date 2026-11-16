mod term_text;

use super::{Expr, parse};
use crate::Categories;
use crate::select::{Command, Filter, FilterTable, Has, TableValue, scope, select_field};
use term_text::{negated_text, term_texts};

pub fn query_table(text: &str, categories: &Categories) -> Option<FilterTable> {
    let expr = parse(text, categories)?;
    let mut table = table_of(&normal(expr, false))?;
    add_defaults(&mut table);
    let filter = Filter::read(&table).ok()?;
    scope(&filter, Command::List).ok()?;
    Some(table)
}

fn normal(expr: Expr, negated: bool) -> Expr {
    match (expr, negated) {
        (Expr::Not(inner), negated) => normal(*inner, !negated),
        (Expr::And(a, b), false) => Expr::And(bx(normal(*a, false)), bx(normal(*b, false))),
        (Expr::Or(a, b), false) => Expr::Or(bx(normal(*a, false)), bx(normal(*b, false))),
        (Expr::And(a, b), true) => Expr::Or(bx(normal(*a, true)), bx(normal(*b, true))),
        (Expr::Or(a, b), true) => Expr::And(bx(normal(*a, true)), bx(normal(*b, true))),
        (Expr::Term(term), false) => Expr::Term(term),
        (Expr::Term(term), true) => Expr::Not(bx(Expr::Term(term))),
    }
}

fn bx(expr: Expr) -> Box<Expr> {
    Box::new(expr)
}

fn table_of(expr: &Expr) -> Option<FilterTable> {
    let mut table = FilterTable::default();
    for part in joined(expr, true) {
        match part {
            Expr::Or(..) if table.any.is_empty() => {
                table.any = joined(part, false)
                    .into_iter()
                    .map(table_of)
                    .collect::<Option<_>>()?
            }
            Expr::Or(..) => return None,
            Expr::Term(term) => add_all(&mut table, term_texts(term)),
            Expr::Not(inner) => match inner.as_ref() {
                Expr::Term(term) => add_all(&mut table, vec![negated_text(term)?]),
                _ => return None,
            },
            Expr::And(..) => return None,
        }
    }
    Some(table)
}

fn joined(expr: &Expr, and: bool) -> Vec<&Expr> {
    match (expr, and) {
        (Expr::And(a, b), true) | (Expr::Or(a, b), false) => {
            let mut parts = joined(a, and);
            parts.extend(joined(b, and));
            parts
        }
        (expr, _) => vec![expr],
    }
}

fn add_all(table: &mut FilterTable, texts: Vec<(&'static str, String)>) {
    for (key, text) in texts {
        let value = match table.fields.remove(key) {
            None => TableValue::One(text),
            Some(held) => TableValue::Many(held.texts().iter().cloned().chain([text]).collect()),
        };
        table.fields.insert(key.to_string(), value);
    }
}

fn add_defaults(table: &mut FilterTable) {
    let plain_of = |has: Has| {
        table.fields.iter().any(|(key, value)| {
            let field = select_field(key);
            key != "done"
                && field.is_some_and(|f| f.has == has)
                && value.texts().iter().all(|t| !t.starts_with("not:"))
        })
    };
    let one_kind = plain_of(Has::Task) || plain_of(Has::Event);
    let done_told = table.fields.contains_key("done")
        || plain_of(Has::Event)
        || table.fields.get("completed_at").is_some_and(|v| {
            v.texts()
                .iter()
                .all(|t| t != "none" && !t.starts_with("not:"))
        });
    if !one_kind && !table.fields.contains_key("kind") {
        table
            .fields
            .insert("kind".into(), TableValue::One("any".into()));
    }
    if !done_told {
        table
            .fields
            .insert("done".into(), TableValue::One("any".into()));
    }
}

#[cfg(test)]
mod tests;
