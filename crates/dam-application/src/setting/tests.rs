use super::{BadValue, KeyRefusal, SettingKey, SettingKind, SettingValue};

fn path(key: &str) -> Vec<String> {
    SettingKey::parse(key).unwrap().path
}

#[test]
fn a_key_splits_into_its_group_its_name_and_its_field() {
    assert_eq!(path("owner"), ["owner"]);
    assert_eq!(path("done.interactive"), ["done", "interactive"]);
    assert_eq!(
        path("remote.todoist.deadline"),
        ["remote", "todoist", "deadline"]
    );
    assert_eq!(path("filter.a.b.query"), ["filter", "a.b", "query"]);
}

#[test]
fn a_secret_a_function_and_an_unknown_key_are_refused() {
    let refused = |key: &str| SettingKey::parse(key).unwrap_err();
    assert_eq!(
        refused("token.ci.value"),
        KeyRefusal::Secret("token.ci.value".into())
    );
    assert_eq!(
        refused("remote.todoist.api_token"),
        KeyRefusal::Secret("remote.todoist.api_token".into())
    );
    assert_eq!(
        refused("filter.today.match"),
        KeyRefusal::Function("filter.today.match".into())
    );
    assert_eq!(
        refused("hook.task_new"),
        KeyRefusal::Function("hook.task_new".into())
    );
    assert_eq!(
        refused("remote.todoist"),
        KeyRefusal::Unknown("remote.todoist".into())
    );
    assert_eq!(refused("colour"), KeyRefusal::Unknown("colour".into()));
}

#[test]
fn a_value_is_read_as_the_type_of_its_setting() {
    let value = |key: &str, text: &str| SettingKey::parse(key).unwrap().value(text);
    assert_eq!(
        value("done.interactive", "true"),
        Ok(SettingValue::Flag(true))
    );
    assert_eq!(value("progress", "false"), Ok(SettingValue::Flag(false)));
    assert_eq!(
        value("remote.t.deadline", "15m"),
        Ok(SettingValue::Text("15m".into()))
    );
    assert_eq!(
        value("category.p.values", "a, b,c"),
        Ok(SettingValue::List(vec!["a".into(), "b".into(), "c".into()]))
    );
    assert_eq!(
        value("remote.t.deadline", "soon"),
        Err(BadValue {
            key: "remote.t.deadline".into(),
            value: "soon".into(),
            expected: SettingKind::Duration
        })
    );
    assert!(value("category.p.exclusive", "yes").is_err());
}
