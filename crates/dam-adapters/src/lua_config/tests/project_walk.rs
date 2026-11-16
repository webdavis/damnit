use std::sync::mpsc;
use std::time::Duration;

use dam_application::ConfigError;

use super::project_support::ProjectFolder;

fn refused_within_a_second(text: &str) -> ConfigError {
    let folder = ProjectFolder::with(text);
    let file = folder.file();
    let (sender, received) = mpsc::channel();
    std::thread::spawn(move || {
        let _ = sender.send(folder.read());
    });
    let read = received
        .recv_timeout(Duration::from_secs(1))
        .expect("reading the file ran past a second");
    let refused = read.expect_err("the file was read");
    let named = format!("{refused:?}");
    assert!(named.contains(&*file.to_string_lossy()), "{named}");
    refused
}

#[test]
fn a_table_whose_entries_all_share_one_child_is_refused_quickly() {
    refused_within_a_second(
        "local t = {} \
         for _ = 1, 6 do local n = {} for i = 1, 50 do n['k' .. i] = t end t = n end \
         return { path = 'a', ls = t }",
    );
    refused_within_a_second(
        "local t = { 'x' } \
         for _ = 1, 6 do local n = {} for i = 1, 50 do n[i] = t end t = n end \
         return { path = 'a', new_task = { labels = t } }",
    );
}

#[test]
fn a_table_that_holds_itself_is_refused_quickly() {
    refused_within_a_second("local t = {}; t.t = t; return { path = 'a', ls = t }");
    refused_within_a_second("local t = {}; t.t = t; return { path = 'a', filter = { mine = t } }");
}

#[test]
fn a_list_of_a_million_entries_is_refused_quickly() {
    refused_within_a_second(
        "local l = {} for i = 1, 1000000 do l[i] = 'x' end \
         return { path = 'a', new_task = { labels = l } }",
    );
    refused_within_a_second(
        "local l = {} for i = 1, 1000000 do l[i] = 'due' end \
         return { path = 'a', required = l }",
    );
}
