use crate::support;

use support::sandbox::Sandbox;

#[test]
fn hook_list_reads_each_hook_file_and_names_the_one_that_does_not_load() {
    let _guard =
        support::guard("hook_list_reads_each_hook_file_and_names_the_one_that_does_not_load");
    let sb = Sandbox::new();
    sb.write_hooks(&[
        (
            "task_new",
            "{ run = function() end, on_failure = 'refuse' }",
        ),
        ("task_done", "'notify'"),
    ]);
    let (ok, out, err) = sb.dam(&["hook", "list"]);
    assert!(ok, "{err}");
    assert_eq!(
        out,
        "task_new   hooks/task_new.lua   on_failure     refuse\n\
         task_done  hooks/task_done.lua  does not load\n"
    );
    assert_eq!(
        err,
        "warning: hooks/task_done.lua must return a function, or a table holding one as run, \
         and it returned a string\n"
    );
}
