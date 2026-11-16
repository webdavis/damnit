use std::collections::BTreeSet;
use std::fmt;

use dam_protocol::{
    WireAttachment, WireAttendee, WireConference, WireEvent, WireFocus, WireObject, WirePerson,
    WireTask,
};
use serde::de::{self, Deserialize, Deserializer, Visitor};
use serde_json::Value;

use super::filter_support::{every_field_event, every_field_task};
use crate::object_document;

#[derive(Debug)]
struct Named(Vec<&'static str>);

impl fmt::Display for Named {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}", self.0)
    }
}

impl std::error::Error for Named {}

impl de::Error for Named {
    fn custom<T: fmt::Display>(_: T) -> Self {
        Named(vec![])
    }
}

struct FieldNames;

impl<'de> Deserializer<'de> for FieldNames {
    type Error = Named;

    fn deserialize_any<V: Visitor<'de>>(self, _: V) -> Result<V::Value, Named> {
        Err(Named(vec![]))
    }

    fn deserialize_struct<V: Visitor<'de>>(
        self,
        _: &'static str,
        fields: &'static [&'static str],
        _: V,
    ) -> Result<V::Value, Named> {
        Err(Named(fields.to_vec()))
    }

    serde::forward_to_deserialize_any! {
        bool i8 i16 i32 i64 i128 u8 u16 u32 u64 u128 f32 f64 char str string bytes byte_buf
        option unit unit_struct newtype_struct seq tuple tuple_struct map enum identifier
        ignored_any
    }
}

fn fields_of<'de, T: Deserialize<'de>>() -> BTreeSet<&'static str> {
    match T::deserialize(FieldNames) {
        Err(Named(fields)) => fields.into_iter().collect(),
        Ok(_) => BTreeSet::new(),
    }
}

fn keys(documents: &[&Value]) -> BTreeSet<String> {
    documents
        .iter()
        .filter_map(|document| document.as_object())
        .flat_map(|map| map.keys().cloned())
        .collect()
}

type Level<'a> = (
    &'a str,
    BTreeSet<&'static str>,
    Vec<&'a Value>,
    &'a [&'a str],
);

#[test]
fn the_fixtures_set_every_field_of_the_object_document() {
    let task = serde_json::to_value(object_document(&every_field_task())).unwrap();
    let event = serde_json::to_value(object_document(&every_field_event())).unwrap();
    let (t, e) = (&task["task"], &event["event"]);
    let levels: [Level; 8] = [
        (
            "object",
            fields_of::<WireObject>(),
            vec![&task, &event],
            &["remote_id", "container_remote_id"],
        ),
        ("task", fields_of::<WireTask>(), vec![t], &["event"]),
        ("focus", fields_of::<WireFocus>(), vec![&t["focus"]], &[]),
        ("event", fields_of::<WireEvent>(), vec![e], &[]),
        (
            "attendee",
            fields_of::<WireAttendee>(),
            vec![&e["attendees"][0]],
            &[],
        ),
        (
            "organizer",
            fields_of::<WirePerson>(),
            vec![&e["organizer"]],
            &[],
        ),
        (
            "conference",
            fields_of::<WireConference>(),
            vec![&e["conference"]],
            &[],
        ),
        (
            "attachment",
            fields_of::<WireAttachment>(),
            vec![&e["attachments"][0]],
            &[],
        ),
    ];
    for (level, fields, documents, never_in_the_document) in levels {
        assert!(!fields.is_empty(), "{level}");
        let set = keys(&documents);
        let unset: Vec<_> = fields
            .into_iter()
            .filter(|field| !set.contains(*field) && !never_in_the_document.contains(field))
            .collect();
        assert!(
            unset.is_empty(),
            "the fixture leaves {level} fields unset: {unset:?}"
        );
    }
}
