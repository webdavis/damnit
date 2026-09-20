use dam_protocol::WireObject;

use crate::map::api_priority;

/// How many commands one mutation may become. The uuid of each is the
/// mutation's key with its last character replaced by the command's ordinal,
/// so the ordinal has one hex digit to fit in.
const MAX_COMMANDS: usize = 16;

/// The shape dam's `idempotency_key` arrives in: a UUID whose last character
/// dam leaves free for the ordinal.
const KEY_LEN: usize = 36;

/// The uuid for command `n` of a mutation.
///
/// Todoist deduplicates by it: "Todoist will not execute a command that has
/// same UUID as a previously executed command." So every command of a resent
/// mutation has to repeat the uuid its first attempt used, which is why the
/// number comes from the ordinal rather than from anything minted here.
pub fn uuid_for(key: &str, n: usize) -> Result<String, String> {
    if key.len() != KEY_LEN
        || !key.bytes().enumerate().all(|(i, b)| {
            if matches!(i, 8 | 13 | 18 | 23) {
                b == b'-'
            } else {
                b.is_ascii_hexdigit()
            }
        })
    {
        return Err(format!(
            "an idempotency key is a {KEY_LEN}-character UUID, got {key:?}"
        ));
    }
    if n >= MAX_COMMANDS {
        return Err(format!("a mutation takes at most {MAX_COMMANDS} commands"));
    }
    Ok(format!("{}{:x}", &key[..KEY_LEN - 1], n))
}

/// One Sync API command.
pub fn command(kind: &str, uuid: String, args: serde_json::Value) -> serde_json::Value {
    serde_json::json!({ "type": kind, "uuid": uuid, "args": args })
}

/// A command that creates an object, with `temp_id` standing in for the id
/// Todoist will issue. The real id comes back in the answer's
/// `temp_id_mapping` under this same placeholder.
pub fn creating_command(
    kind: &str,
    uuid: String,
    temp_id: &str,
    args: serde_json::Value,
) -> serde_json::Value {
    serde_json::json!({ "type": kind, "uuid": uuid, "temp_id": temp_id, "args": args })
}

/// `item_add` or `item_update` args for `object`. `only` is the changed-field
/// list on an update and `None` on a create, which is also what decides
/// whether an absent date is omitted or sent as `null` to clear one.
pub fn item_args(object: &WireObject, only: Option<&[String]>) -> serde_json::Value {
    let want = |f: &str| only.is_none_or(|fields| fields.iter().any(|x| x == f));
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
    // A create has no prior deadline to clear, so an absent one is omitted
    // rather than sent as the null that clears one.
    if want("deadline") {
        match (&task.deadline, only) {
            (Some(date), _) => {
                args.insert("deadline".into(), serde_json::json!({ "date": date }));
            }
            (None, Some(_)) => {
                args.insert("deadline".into(), serde_json::Value::Null);
            }
            (None, None) => {}
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
            uuid_for(KEY, 0).unwrap(),
            "0102030a-0b0c-4d0e-8f10-111213141500"
        );
        assert_eq!(
            uuid_for(KEY, 2).unwrap(),
            "0102030a-0b0c-4d0e-8f10-111213141502"
        );
        assert_ne!(uuid_for(KEY, 0).unwrap(), uuid_for(KEY, 1).unwrap());
    }

    #[test]
    fn a_key_that_is_not_a_uuid_is_refused_rather_than_sliced() {
        for bad in ["", "short", &"x".repeat(36), &"0".repeat(36)] {
            assert!(uuid_for(bad, 0).is_err(), "{bad:?}");
        }
        assert!(uuid_for(KEY, MAX_COMMANDS).is_err());
    }
}
