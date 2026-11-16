use super::date::DateTest;
use super::field::{FieldType, Holds, SelectField};
use super::want::Want;
use crate::{Oid, Path, parse_minutes};

pub(crate) fn wants(field: &SelectField, text: &str) -> Option<Vec<Want>> {
    if let Some(exact) = text.strip_prefix('=') {
        return one(field.ty, exact).map(|w| vec![w]);
    }
    if let FieldType::Category = field.ty {
        return category(text);
    }
    if !field.takes_a_list() {
        return alternative(field, text).map(|w| vec![w]);
    }
    text.split(',')
        .map(|part| alternative(field, part))
        .collect()
}

fn alternative(field: &SelectField, text: &str) -> Option<Want> {
    match text {
        "none" if field.holds != Holds::One => Some(Want::Empty),
        text => one(field.ty, text),
    }
}

fn one(ty: FieldType, text: &str) -> Option<Want> {
    if text.is_empty() {
        return None;
    }
    match ty {
        FieldType::Text => Some(Want::Contains(text.to_lowercase())),
        FieldType::Choice(choices) => choices().contains(&text).then(|| match text {
            "any" => Want::Any,
            text => Want::Is(text.into()),
        }),
        FieldType::Number(lo, hi) => {
            range(text, |s| s.parse().ok().filter(|n| (lo..=hi).contains(n)))
        }
        FieldType::Length => range(text, |s| parse_minutes(s).map(i64::from)),
        FieldType::TextSet => Some(Want::Is(text.into())),
        FieldType::Path => Path::parse(text).ok().map(Want::Under),
        FieldType::Reference => match text.strip_prefix("in:") {
            Some(filter) => one(FieldType::FilterName, filter),
            None => Oid::parse(text).ok().map(Want::Id),
        },
        FieldType::YesNo => match text {
            "true" => Some(Want::Yes(true)),
            "false" => Some(Want::Yes(false)),
            "any" => Some(Want::Any),
            _ => None,
        },
        FieldType::Date => DateTest::read(text).map(Want::Date),
        FieldType::Category => category(text)?.pop(),
        FieldType::FilterName => Some(Want::ListedBy(text.into())),
    }
}

fn range(text: &str, number: impl Fn(&str) -> Option<i64>) -> Option<Want> {
    let end = |side: &str| match side {
        "" => Some(None),
        side => number(side).map(Some),
    };
    match text.split_once("..") {
        None => number(text).map(|n| Want::Range(Some(n), Some(n))),
        Some(("", "")) => None,
        Some((from, to)) => Some(Want::Range(end(from)?, end(to)?)),
    }
}

fn category(text: &str) -> Option<Vec<Want>> {
    let nonempty = |s: &str| (!s.is_empty()).then(|| s.to_string());
    match text.split_once('=') {
        Some((group, labels)) => labels
            .split(',')
            .map(|label| Some(Want::Group(nonempty(group)?, Some(nonempty(label)?))))
            .collect(),
        None => text
            .split(',')
            .map(|group| Some(Want::Group(nonempty(group)?, None)))
            .collect(),
    }
}
