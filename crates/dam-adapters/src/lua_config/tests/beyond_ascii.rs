use dam_application::RemoteName;

use super::support::config;

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
