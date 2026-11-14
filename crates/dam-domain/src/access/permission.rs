use crate::Kind;
use crate::vocabulary::vocabulary;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum Permission {
    TaskRead,
    TaskCreate,
    TaskChange,
    TaskComplete,
    TaskMove,
    TaskRemove,
    EventRead,
    EventCreate,
    EventChange,
    EventMove,
    EventRemove,
    PendingRead,
    PendingAccept,
    PendingReject,
    WorkingRead,
    StageRead,
    HistoryRead,
    HistoryCommit,
    RemoteRead,
    RemotePull,
    RemotePush,
    ConflictRead,
    ConflictResolve,
    ConfigRead,
}

vocabulary!(Permission {
    TaskRead => "task:read",
    TaskCreate => "task:create",
    TaskChange => "task:change",
    TaskComplete => "task:complete",
    TaskMove => "task:move",
    TaskRemove => "task:remove",
    EventRead => "event:read",
    EventCreate => "event:create",
    EventChange => "event:change",
    EventMove => "event:move",
    EventRemove => "event:remove",
    PendingRead => "pending:read",
    PendingAccept => "pending:accept",
    PendingReject => "pending:reject",
    WorkingRead => "working:read",
    StageRead => "stage:read",
    HistoryRead => "history:read",
    HistoryCommit => "history:commit",
    RemoteRead => "remote:read",
    RemotePull => "remote:pull",
    RemotePush => "remote:push",
    ConflictRead => "conflict:read",
    ConflictResolve => "conflict:resolve",
    ConfigRead => "config:read",
});

pub const RESERVED_PERMISSIONS: [&str; 4] = [
    "token:create",
    "token:revoke",
    "config:change",
    "history:rewrite",
];

#[derive(Debug, PartialEq, Eq)]
pub enum PermissionName {
    Grantable(Permission),
    Reserved,
    Unknown,
}

impl PermissionName {
    pub fn classify(name: &str) -> PermissionName {
        if let Some(permission) = Permission::parse(name) {
            PermissionName::Grantable(permission)
        } else if RESERVED_PERMISSIONS.contains(&name) {
            PermissionName::Reserved
        } else {
            PermissionName::Unknown
        }
    }
}

impl Permission {
    pub fn read(kind: Kind) -> Permission {
        match kind {
            Kind::Task => Permission::TaskRead,
            Kind::Event => Permission::EventRead,
        }
    }

    pub fn create(kind: Kind) -> Permission {
        match kind {
            Kind::Task => Permission::TaskCreate,
            Kind::Event => Permission::EventCreate,
        }
    }

    pub fn change(kind: Kind) -> Permission {
        match kind {
            Kind::Task => Permission::TaskChange,
            Kind::Event => Permission::EventChange,
        }
    }

    pub fn remove(kind: Kind) -> Permission {
        match kind {
            Kind::Task => Permission::TaskRemove,
            Kind::Event => Permission::EventRemove,
        }
    }

    pub fn kind(&self) -> Option<Kind> {
        match self.as_str().split_once(':').map(|(kind, _)| kind) {
            Some("task") => Some(Kind::Task),
            Some("event") => Some(Kind::Event),
            _ => None,
        }
    }
}

impl std::fmt::Display for Permission {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

#[cfg(test)]
mod tests;
