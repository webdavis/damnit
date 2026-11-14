use std::ops::Range;

use full_moon::LuaVersion;
use full_moon::ast::{Expression, Field, LastStmt, TableConstructor};
use full_moon::node::Node;
use full_moon::tokenizer::{TokenReference, TokenType};

pub(super) struct TableLayout {
    pub(super) open: usize,
    pub(super) close: usize,
    pub(super) entries: Vec<Entry>,
}

pub(super) struct Entry {
    pub(super) key: Option<String>,
    pub(super) span: Range<usize>,
    pub(super) separator: Option<Range<usize>>,
}

pub(super) struct Located {
    pub(super) root: TableLayout,
    pub(super) section: Option<TableLayout>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum Unwritable {
    NotLua,
    RootBuiltInCode,
    SectionBuiltInCode,
}

pub(super) fn locate(text: &str, section: &str) -> Result<Located, Unwritable> {
    let ast = full_moon::parse_fallible(text, LuaVersion::lua54())
        .into_result()
        .map_err(|_| Unwritable::NotLua)?;
    let root = match ast.nodes().last_stmt() {
        Some(LastStmt::Return(r)) if r.returns().len() == 1 => r.returns().iter().next(),
        _ => None,
    };
    let Some(Expression::TableConstructor(root)) = root else {
        return Err(Unwritable::RootBuiltInCode);
    };
    let value = root.fields().iter().filter_map(|field| match field {
        Field::NameKey { key, value, .. } if name_of(key).as_deref() == Some(section) => {
            Some(value)
        }
        Field::ExpressionKey { key, value, .. } if string_key(key).as_deref() == Some(section) => {
            Some(value)
        }
        _ => None,
    });
    let section = match value.last().map(|v| &**v) {
        None => None,
        Some(Expression::TableConstructor(constructor)) => Some(table(constructor)),
        Some(_) => return Err(Unwritable::SectionBuiltInCode),
    };
    Ok(Located {
        root: table(root),
        section,
    })
}

fn table(constructor: &TableConstructor) -> TableLayout {
    let (open, close) = constructor.braces().tokens();
    TableLayout {
        open: end_of(open),
        close: start_of(close),
        entries: constructor
            .fields()
            .pairs()
            .map(|pair| {
                let field = pair.value();
                Entry {
                    key: key_of(field),
                    span: start_of(field)..end_of(field),
                    separator: pair.punctuation().map(|p| start_of(p)..end_of(p)),
                }
            })
            .collect(),
    }
}

fn key_of(field: &Field) -> Option<String> {
    match field {
        Field::NameKey { key, .. } => name_of(key),
        Field::ExpressionKey { key, .. } => string_key(key),
        _ => None,
    }
}

fn name_of(token: &TokenReference) -> Option<String> {
    match token.token_type() {
        TokenType::Identifier { identifier } => Some(identifier.to_string()),
        _ => None,
    }
}

fn string_key(key: &Expression) -> Option<String> {
    let Expression::String(token) = key else {
        return None;
    };
    match token.token_type() {
        TokenType::StringLiteral { literal, .. } if !literal.contains('\\') => {
            Some(literal.to_string())
        }
        _ => None,
    }
}

fn start_of(node: &impl Node) -> usize {
    node.start_position().map_or(0, |p| p.bytes())
}

fn end_of(node: &impl Node) -> usize {
    node.end_position().map_or(0, |p| p.bytes())
}
