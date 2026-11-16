use dam_application::Uneditable;
use full_moon::LuaVersion;
use full_moon::ast::{Expression, Field, LastStmt, TableConstructor};

use super::insert::insert;
use super::locate::{end_of, key_of, start_of, table};
use super::lua_text::table_key;
use super::remove::remove;
use super::section::indent_per_level;

pub(super) fn edit_setting(
    text: &str,
    path: &[String],
    literal: Option<&str>,
) -> Result<Option<String>, Uneditable> {
    let ast = full_moon::parse_fallible(text, LuaVersion::lua54())
        .into_result()
        .map_err(|_| Uneditable::NotEditable)?;
    let root = match ast.nodes().last_stmt() {
        Some(LastStmt::Return(r)) if r.returns().len() == 1 => r.returns().iter().next(),
        _ => None,
    };
    let Some(Expression::TableConstructor(root)) = root else {
        return Err(Uneditable::ReturnsBuiltInCode);
    };
    let per_level = indent_per_level(text, &table(root));
    let mut current: &TableConstructor = root;
    for (depth, segment) in path.iter().enumerate() {
        let found = current
            .fields()
            .iter()
            .enumerate()
            .filter(|(_, field)| key_of(field).as_deref() == Some(segment))
            .last();
        let Some((index, field)) = found else {
            return Ok(literal.map(|literal| {
                insert(text, &table(current), &per_level, |_| {
                    nested(&path[depth..], literal)
                })
            }));
        };
        let value = value_of(field);
        if depth + 1 == path.len() {
            return Ok(Some(match literal {
                Some(literal) => format!(
                    "{}{literal}{}",
                    &text[..start_of(value)],
                    &text[end_of(value)..]
                ),
                None => remove(text, &table(current), index),
            }));
        }
        let Expression::TableConstructor(inner) = value else {
            return Err(Uneditable::TableBuiltInCode);
        };
        current = inner;
    }
    Err(Uneditable::NotEditable)
}

fn value_of(field: &Field) -> &Expression {
    match field {
        Field::NameKey { value, .. } | Field::ExpressionKey { value, .. } => value,
        Field::NoKey(value) => value,
        _ => unreachable!("full_moon adds no other field"),
    }
}

fn nested(path: &[String], literal: &str) -> String {
    match path {
        [last] => format!("{} = {literal}", table_key(last)),
        [first, rest @ ..] => format!("{} = {{ {} }}", table_key(first), nested(rest, literal)),
        [] => literal.to_string(),
    }
}
