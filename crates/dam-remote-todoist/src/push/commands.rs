use dam_protocol::WireObject;

use crate::map::api_priority;

const COMMAND_ORDINALS_IN_ONE_HEX_DIGIT: usize = 16;

const UUID_KEY_LEN: usize = 36;

const UUID_HYPHEN_OFFSETS: [usize; 4] = [8, 13, 18, 23];

pub fn repeatable_command_uuid(key: &str, ordinal: usize) -> Result<String, String> {
    if !is_hyphenated_uuid(key) {
        return Err(format!(
            "an idempotency key is a {UUID_KEY_LEN}-character UUID, got {key:?}"
        ));
    }
    if ordinal >= COMMAND_ORDINALS_IN_ONE_HEX_DIGIT {
        return Err(format!(
            "a mutation takes at most {COMMAND_ORDINALS_IN_ONE_HEX_DIGIT} commands"
        ));
    }
    let key_without_its_free_last_character = &key[..UUID_KEY_LEN - 1];
    Ok(format!("{key_without_its_free_last_character}{ordinal:x}"))
}

fn is_hyphenated_uuid(key: &str) -> bool {
    key.len() == UUID_KEY_LEN
        && key.bytes().enumerate().all(|(i, b)| {
            if UUID_HYPHEN_OFFSETS.contains(&i) {
                b == b'-'
            } else {
                b.is_ascii_hexdigit()
            }
        })
}

pub fn command(kind: &str, uuid: String, args: serde_json::Value) -> serde_json::Value {
    serde_json::json!({ "type": kind, "uuid": uuid, "args": args })
}

pub fn creating_command(
    kind: &str,
    uuid: String,
    temp_id: &str,
    args: serde_json::Value,
) -> serde_json::Value {
    serde_json::json!({ "type": kind, "uuid": uuid, "temp_id": temp_id, "args": args })
}

pub enum ItemWrite<'f> {
    Create,
    Update { changed: &'f [String] },
}

impl ItemWrite<'_> {
    fn touches(&self, field: &str) -> bool {
        match self {
            ItemWrite::Create => true,
            ItemWrite::Update { changed } => changed.iter().any(|x| x == field),
        }
    }
}

pub fn item_args(object: &WireObject, write: ItemWrite) -> serde_json::Value {
    let want = |f: &str| write.touches(f);
    let mut args = serde_json::Map::new();
    if want("subject") {
        args.insert("content".into(), object.subject.clone().into());
    }
    if want("body") {
        args.insert("description".into(), object.body.clone().into());
    }
    if want("labels") {
        args.insert("labels".into(), object.labels.clone().into());
    }
    let Some(task) = &object.task else {
        return serde_json::Value::Object(args);
    };
    if want("priority") {
        args.insert("priority".into(), api_priority(task.priority).into());
    }
    if want("due") || want("recurrence") {
        args.insert(
            "due".into(),
            match (&object.recurrence, &task.due) {
                (Some(rule), _) => serde_json::json!({ "string": rule }),
                (None, Some(date)) => serde_json::json!({ "date": date }),
                (None, None) => serde_json::Value::Null,
            },
        );
    }
    if want("deadline") {
        match (&task.deadline, &write) {
            (Some(date), _) => {
                args.insert("deadline".into(), serde_json::json!({ "date": date }));
            }
            (None, ItemWrite::Update { .. }) => {
                args.insert("deadline".into(), serde_json::Value::Null);
            }
            (None, ItemWrite::Create) => {}
        }
    }
    serde_json::Value::Object(args)
}

#[cfg(test)]
mod tests {
    use super::*;

    const KEY: &str = "0102030a-0b0c-4d0e-8f10-111213141500";

    #[test]
    fn a_uuid_numbers_the_command_in_the_character_dam_left_free() {
        assert_eq!(
            repeatable_command_uuid(KEY, 0).unwrap(),
            "0102030a-0b0c-4d0e-8f10-111213141500"
        );
        assert_eq!(
            repeatable_command_uuid(KEY, 2).unwrap(),
            "0102030a-0b0c-4d0e-8f10-111213141502"
        );
        assert_ne!(
            repeatable_command_uuid(KEY, 0).unwrap(),
            repeatable_command_uuid(KEY, 1).unwrap()
        );
    }

    #[test]
    fn a_key_that_is_not_a_uuid_is_refused_rather_than_sliced() {
        for bad in ["", "short", &"x".repeat(36), &"0".repeat(36)] {
            assert!(repeatable_command_uuid(bad, 0).is_err(), "{bad:?}");
        }
        assert!(repeatable_command_uuid(KEY, COMMAND_ORDINALS_IN_ONE_HEX_DIGIT).is_err());
    }
}
