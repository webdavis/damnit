use std::path::PathBuf;

use dam_domain::CategoryError;

use super::misread::{Misread, Place};
use crate::ports::RemoteNameError;

#[derive(Debug, PartialEq, Eq)]
pub enum ConfigError {
    NotLua(PathBuf),
    Unreadable {
        path: PathBuf,
        why: String,
    },
    Unwritable {
        path: PathBuf,
        why: String,
    },
    StoppedLoading {
        path: PathBuf,
        line: Option<u32>,
        reason: Option<String>,
    },
    NotATable {
        path: PathBuf,
        returned: String,
    },
    NotNamedSettings {
        file: String,
    },
    HookMoved {
        config: String,
        events: Vec<String>,
    },
    UnknownHookFile {
        file: String,
    },
    NotAHook {
        file: String,
        returned: String,
    },
    Misread(Box<Misread>),
    BelongsInSecrets {
        key: String,
        at: Place,
    },
    BelongsInPersonalConfig {
        key: String,
        file: PathBuf,
    },
    HoldsAControlCharacter {
        file: PathBuf,
    },
    NotAPath {
        file: PathBuf,
    },
    BadRemoteName(RemoteNameError),
    BadCategories(CategoryError),
    UndeclaredRemote {
        remote: String,
        config: String,
    },
    CannotEdit {
        table: EditedTable,
        why: Uneditable,
        by_hand: ByHand,
    },
    ReadOnly {
        table: EditedTable,
        by_hand: ByHand,
    },
    AlreadyHeld {
        table: EditedTable,
        entry: String,
    },
    NotHeld {
        table: EditedTable,
        entry: String,
    },
    HeldUnderUneditableKey {
        table: EditedTable,
        entry: String,
        by_hand: ByHand,
    },
    EditWouldBreak {
        table: EditedTable,
        entry: String,
        fault: EditFault,
    },
    SettingNotEditable {
        key: String,
        why: Uneditable,
        by_hand: String,
    },
    SettingEditWouldBreak {
        key: String,
    },
    SettingReadOnly {
        key: String,
        by_hand: String,
    },
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum EditedTable {
    Remotes,
    Tokens,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Uneditable {
    NotEditable,
    ReturnsBuiltInCode,
    TableBuiltInCode,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum EditFault {
    WouldNotHoldTheEntry,
    WouldStillHoldTheEntry,
    WouldNotKeepTheOthers,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ByHand {
    AddRemote { entry: String },
    AddToken { name: String },
    RemoveToken { name: String },
}

impl From<Misread> for ConfigError {
    fn from(misread: Misread) -> ConfigError {
        ConfigError::Misread(Box::new(misread))
    }
}
