use super::*;

#[test]
fn every_permission_name_is_kind_colon_action_and_reads_back() {
    for permission in Permission::ALL {
        let name = permission.as_str();
        let (kind, action) = name.split_once(':').unwrap();
        assert!(!kind.is_empty() && !action.is_empty(), "{name}");
        assert_eq!(Permission::parse(name), Some(permission));
    }
}

#[test]
fn shipped_names_keep_their_spelling() {
    let names: Vec<&str> = Permission::ALL.iter().map(Permission::as_str).collect();
    assert_eq!(
        names,
        [
            "task:read",
            "task:create",
            "task:change",
            "task:complete",
            "task:move",
            "task:remove",
            "event:read",
            "event:create",
            "event:change",
            "event:move",
            "event:remove",
            "pending:read",
            "pending:accept",
            "pending:reject",
            "working:read",
            "stage:read",
            "history:read",
            "history:commit",
            "remote:read",
            "remote:pull",
            "remote:push",
            "conflict:read",
            "conflict:resolve",
            "config:read",
        ]
    );
}

#[test]
fn a_name_is_grantable_reserved_or_unknown() {
    assert_eq!(
        PermissionName::classify("task:read"),
        PermissionName::Grantable(Permission::TaskRead)
    );
    for reserved in RESERVED_PERMISSIONS {
        assert_eq!(PermissionName::classify(reserved), PermissionName::Reserved);
    }
    for unknown in ["task:frobnicate", "task:*", "*", "task", "TASK:READ", ""] {
        assert_eq!(PermissionName::classify(unknown), PermissionName::Unknown);
    }
}

#[test]
fn only_minting_tokens_editing_config_and_rewriting_history_are_reserved() {
    assert_eq!(
        RESERVED_PERMISSIONS,
        [
            "token:create",
            "token:revoke",
            "config:change",
            "history:rewrite"
        ]
    );
    for grantable in [
        "conflict:resolve",
        "history:commit",
        "pending:accept",
        "pending:reject",
    ] {
        assert!(Permission::parse(grantable).is_some(), "{grantable}");
    }
}

#[test]
fn no_name_is_both_grantable_and_reserved() {
    for reserved in RESERVED_PERMISSIONS {
        assert_eq!(Permission::parse(reserved), None, "{reserved}");
    }
}

#[test]
fn task_and_event_permissions_name_their_kind_and_the_rest_name_none() {
    assert_eq!(Permission::TaskMove.kind(), Some(Kind::Task));
    assert_eq!(Permission::EventRead.kind(), Some(Kind::Event));
    assert_eq!(Permission::WorkingRead.kind(), None);
    assert_eq!(Permission::read(Kind::Event), Permission::EventRead);
}

#[test]
fn each_kind_names_its_own_create_and_change() {
    use crate::Kind;
    assert_eq!(Permission::create(Kind::Task), Permission::TaskCreate);
    assert_eq!(Permission::create(Kind::Event), Permission::EventCreate);
    assert_eq!(Permission::change(Kind::Task), Permission::TaskChange);
    assert_eq!(Permission::change(Kind::Event), Permission::EventChange);
}
