use dam_protocol::WireObject;
use serde::Serialize;

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

pub type Args = serde_json::Map<String, serde_json::Value>;

#[derive(Debug, Serialize)]
pub struct Command {
    #[serde(rename = "type")]
    pub kind: &'static str,
    pub uuid: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub temp_id: Option<String>,
    pub args: Args,
}

pub fn command(kind: &'static str, uuid: String, args: Args) -> Command {
    Command {
        kind,
        uuid,
        temp_id: None,
        args,
    }
}

pub fn creating_command(kind: &'static str, uuid: String, temp_id: &str, args: Args) -> Command {
    Command {
        temp_id: Some(temp_id.to_string()),
        ..command(kind, uuid, args)
    }
}

pub fn args<const N: usize>(pairs: [(&str, serde_json::Value); N]) -> Args {
    pairs
        .into_iter()
        .map(|(key, value)| (key.to_string(), value))
        .collect()
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

pub fn item_args(object: &WireObject, write: ItemWrite) -> Args {
    let want = |f: &str| write.touches(f);
    let mut args = Args::new();
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
        return args;
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
    args
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
