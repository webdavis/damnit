mod support;

use std::os::unix::fs::PermissionsExt;
use std::path::Path;

use support::sandbox::Sandbox;
use support::token_table::TokenTable;

fn mark_read_only(path: &Path) -> String {
    std::fs::set_permissions(path, std::fs::Permissions::from_mode(0o400)).unwrap();
    std::fs::read_to_string(path).unwrap()
}

fn assert_untouched(path: &Path, before: &str) {
    assert_eq!(std::fs::read_to_string(path).unwrap(), before);
    let mode = std::fs::metadata(path).unwrap().permissions().mode() & 0o777;
    assert_eq!(mode, 0o400);
}

fn refused(sb: &Sandbox, line: &[&str]) -> String {
    let (ok, out, err) = sb.dam(line);
    assert!(!ok && out.is_empty(), "{line:?}: {out}{err}");
    assert!(!err.contains("damk_"), "{err}");
    err
}

#[test]
fn a_token_command_leaves_a_secrets_lua_marked_read_only_as_it_is() {
    let _guard = support::guard("a_token_command_leaves_a_secrets_lua_marked_read_only_as_it_is");
    let sb = Sandbox::new();
    sb.add_token("tool", TokenTable::valued("not-a-token"));
    let before = mark_read_only(&sb.secrets_path());

    let err = refused(&sb, &["token", "create", "other", "--allow", "task:read"]);
    assert_eq!(
        err,
        "dam: config: secrets.lua is read-only; make it writable, or add token other to its \
         token table by hand\n"
    );
    let err = refused(&sb, &["token", "revoke", "tool"]);
    assert_eq!(
        err,
        "dam: config: secrets.lua is read-only; make it writable, or remove token tool from it \
         by hand\n"
    );
    assert_untouched(&sb.secrets_path(), &before);
}

#[test]
fn adding_a_remote_leaves_a_config_lua_marked_read_only_as_it_is() {
    let _guard = support::guard("adding_a_remote_leaves_a_config_lua_marked_read_only_as_it_is");
    let sb = Sandbox::new();
    let before = mark_read_only(&sb.config_path());
    let err = refused(&sb, &["remote", "add", "work", "work::"]);
    assert_eq!(
        err,
        "dam: config: config.lua is read-only; make it writable, or add the remote to its remote \
         table by hand: work = { url = \"work::\" }\n"
    );
    assert_untouched(&sb.config_path(), &before);
}
