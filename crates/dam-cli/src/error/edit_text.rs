use std::fmt;

use dam_application::{ByHand, EditFault, EditedTable, Uneditable};

pub(super) fn named(table: EditedTable) -> (&'static str, &'static str) {
    match table {
        EditedTable::Remotes => ("config.lua", "remote"),
        EditedTable::Tokens => ("secrets.lua", "token"),
    }
}

pub(super) fn cannot_edit(
    f: &mut fmt::Formatter<'_>,
    table: EditedTable,
    why: Uneditable,
    by_hand: &ByHand,
) -> fmt::Result {
    let (file, table) = named(table);
    match why {
        Uneditable::NotEditable => write!(f, "{file} has Lua that dam cannot edit"),
        Uneditable::ReturnsBuiltInCode => write!(f, "{file} returns a table built in code"),
        Uneditable::TableBuiltInCode => write!(f, "the {table} table in {file} is built in code"),
    }?;
    write!(
        f,
        ", and dam edits only a table written out in full. {}",
        ByHandText(by_hand)
    )
}

pub(super) fn would_break(
    f: &mut fmt::Formatter<'_>,
    table: EditedTable,
    entry: &str,
    fault: EditFault,
) -> fmt::Result {
    let (file, table) = named(table);
    write!(f, "the new {file} ")?;
    match fault {
        EditFault::WouldNotHoldTheEntry => write!(f, "would not hold {table} {entry}"),
        EditFault::WouldStillHoldTheEntry => write!(f, "would still hold {table} {entry}"),
        EditFault::WouldNotKeepTheOthers => write!(f, "would not keep every other {table}"),
    }
}

pub(super) struct ByHandText<'a>(pub(super) &'a ByHand);

impl fmt::Display for ByHandText<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.0 {
            ByHand::AddRemote { entry } => {
                write!(f, "Add the remote to its remote table by hand: {entry}")
            }
            by_hand @ ByHand::AddToken { .. } => write!(
                f,
                "Write the table out in full and run the command again, or {}.",
                ByHandClause(by_hand)
            ),
            ByHand::RemoveToken { name } => write!(f, "Remove token {name} from it by hand."),
        }
    }
}

pub(super) struct ByHandClause<'a>(pub(super) &'a ByHand);

impl fmt::Display for ByHandClause<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.0 {
            ByHand::AddRemote { entry } => {
                write!(f, "add the remote to its remote table by hand: {entry}")
            }
            ByHand::AddToken { name } => write!(f, "add token {name} to its token table by hand"),
            ByHand::RemoveToken { name } => write!(f, "remove token {name} from it by hand"),
        }
    }
}
