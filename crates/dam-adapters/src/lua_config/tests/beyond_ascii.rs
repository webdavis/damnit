use dam_application::RemoteName;

use super::support::{bare_token, config, secrets};

#[test]
fn a_remote_lands_in_place_after_text_beyond_ascii() {
    let before = "-- café, 日本\nreturn {\n  -- 名前\n  remote = {\n    work = { url = \"work::\" },\n  },\n}\n";
    let folder = config(before);
    let todoist = RemoteName::parse("todoist").unwrap();
    folder.file().add_remote(&todoist, "todoist::").unwrap();
    assert_eq!(
        folder.text("config.lua"),
        before.replace(
            "    work = { url = \"work::\" },\n",
            "    work = { url = \"work::\" },\n    todoist = { url = \"todoist::\" },\n"
        )
    );
}

#[test]
fn a_token_lands_in_place_after_text_beyond_ascii() {
    let before = "-- café, 日本\nreturn {\n  -- 名前\n  token = {\n    laptop = { value = \"x\" },\n  },\n}\n";
    let folder = secrets(before);
    let token = bare_token("tool");
    folder.file().add_token(&token).unwrap();
    let value = token.value.expose();
    assert_eq!(
        folder.text("secrets.lua"),
        before.replace(
            "    laptop = { value = \"x\" },\n",
            &format!(
                "    laptop = {{ value = \"x\" }},\n    tool = {{\n      value = \"{value}\",\n      \
                 grant = {{\n        {{ allow = {{ \"task:read\" }} }},\n      }},\n    }},\n"
            )
        )
    );
}
