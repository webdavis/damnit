use std::path::PathBuf;

use dam_domain::CategoryError;

use super::edit_kinds::{ByHand, EditFault, EditedTable, Uneditable};
use super::misread::{Misread, Place};
use super::project_rule::ProjectRuleFault;
use crate::ports::RemoteNameError;
use crate::use_cases::selected::SelectError;

#[derive(Clone, Debug, PartialEq, Eq)]
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
    NeedsANewerDam {
        file: PathBuf,
        needs: String,
        this: String,
    },
    QueryText {
        file: String,
        was: String,
        key: String,
        table: Option<String>,
    },
    FilterTable {
        file: String,
        key: String,
        error: Box<SelectError>,
    },
    FilterReachesItself {
        file: String,
        chain: Vec<String>,
    },
    ProjectRule {
        file: PathBuf,
        key: &'static str,
        fault: ProjectRuleFault,
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
    ProjectSettingWouldNotLoad {
        key: String,
        why: Box<ConfigError>,
    },
}

impl From<Misread> for ConfigError {
    fn from(misread: Misread) -> ConfigError {
        ConfigError::Misread(Box::new(misread))
    }
}
