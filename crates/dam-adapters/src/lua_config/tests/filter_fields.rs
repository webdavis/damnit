use std::collections::BTreeSet;

use serde_json::Value;

use super::filter_support::{every_field_event, every_field_task, field_names, run_filter};
use crate::object_document;

pub(super) const NOT_IN_THE_TABLE: [&str; 3] = ["author", "details", "id"];

const WRITE_FIELD_NAMES: &str = r#"
        local names = {}
        local function walk(t, prefix)
          for key, value in pairs(t) do
            local name = prefix == "" and key or prefix .. "." .. key
            if type(value) == "table" and type(value[1]) == "table" then
              for _, item in ipairs(value) do walk(item, name .. "[]") end
            elseif type(value) == "table" and value[1] == nil and next(value) ~= nil then
              walk(value, name)
            else
              names[#names + 1] = name
            end
          end
        end
        walk(task, "")
        table.sort(names)
        local file = io.open(PATH, "w")
        file:write(table.concat(names, "\n"))
        file:close()
        return true
"#;

fn schema_field_names() -> BTreeSet<String> {
    let schema: Value = serde_json::from_str(include_str!(
        "../../../../../docs/schemas/object.schema.json"
    ))
    .unwrap();
    let mut names = BTreeSet::new();
    schema_walk(&schema, &schema, "", &mut names);
    names
        .into_iter()
        .filter(|name| !NOT_IN_THE_TABLE.contains(&name.as_str()))
        .collect()
}

fn resolved<'a>(schema: &'a Value, node: &'a Value) -> &'a Value {
    match node["$ref"]
        .as_str()
        .and_then(|r| r.strip_prefix("#/$defs/"))
    {
        Some(name) => &schema["$defs"][name],
        None => node,
    }
}

fn schema_walk(schema: &Value, node: &Value, prefix: &str, names: &mut BTreeSet<String>) {
    let Some(properties) = resolved(schema, node)["properties"].as_object() else {
        return;
    };
    for (key, value) in properties {
        let name = match prefix {
            "" => key.clone(),
            _ => format!("{prefix}.{key}"),
        };
        let value = resolved(schema, value);
        let items = resolved(schema, &value["items"]);
        if value["properties"].is_object() {
            schema_walk(schema, value, &name, names);
        } else if items["properties"].is_object() {
            schema_walk(schema, items, &format!("{name}[]"), names);
        } else {
            names.insert(name);
        }
    }
}

fn names_match_was_handed(object: &dam_domain::Object) -> BTreeSet<String> {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("names");
    let body = WRITE_FIELD_NAMES.replace("PATH", &format!("{:?}", path.display().to_string()));
    run_filter(&body, std::slice::from_ref(object)).unwrap();
    std::fs::read_to_string(path)
        .unwrap()
        .lines()
        .map(str::to_string)
        .collect()
}

#[test]
fn match_is_handed_every_field_a_task_and_an_event_carry_named_as_the_schema_names_them() {
    let task = every_field_task();
    let event = every_field_event();
    let handed: BTreeSet<String> = names_match_was_handed(&task)
        .into_iter()
        .chain(names_match_was_handed(&event))
        .collect();
    assert_eq!(handed, schema_field_names());
}

#[test]
fn match_is_handed_the_document_dam_show_prints() {
    let task = every_field_task();
    let document = serde_json::to_value(object_document(&task)).unwrap();
    assert_eq!(names_match_was_handed(&task), field_names(&document));
}

#[test]
fn match_is_told_which_kind_it_was_handed() {
    let (task, event) = (every_field_task(), every_field_event());
    let kept = run_filter(
        "        return task.kind == 'event'",
        &[task.clone(), event.clone()],
    );
    assert_eq!(kept.unwrap(), [false, true]);
}

#[test]
fn writing_to_the_task_changes_nothing_damnit_holds() {
    let task = every_field_task();
    let before = task.clone();
    let kept = run_filter(
        "        task.subject = 'changed'\n        task.task.priority = 4\n        return true",
        &[task.clone(), task.clone()],
    );
    assert_eq!(kept.unwrap(), [true, true]);
    assert_eq!(task, before);
    let second_sees_a_fresh_copy = run_filter(
        "        local fresh = task.subject == 'report'\n        task.subject = 'changed'\n        return fresh",
        &[task.clone(), task.clone()],
    );
    assert_eq!(second_sees_a_fresh_copy.unwrap(), [true, true]);
}
