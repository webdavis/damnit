use dam_application::{ByHand, ConfigError, EditFault, EditedTable, Uneditable};

use super::text;

fn tool() -> (EditedTable, String) {
    (EditedTable::Tokens, "tool".into())
}

#[test]
fn an_edit_dam_cannot_make_says_what_to_write_by_hand() {
    let cannot_edit = |why| {
        let (table, name) = tool();
        ConfigError::CannotEdit {
            table,
            why,
            by_hand: ByHand::AddToken { name },
        }
    };
    let add_by_hand = "and dam edits only a table written out in full. Write the table out in \
                       full and run the command again, or add token tool to its token table by \
                       hand.";
    for (why, place) in [
        (
            Uneditable::NotEditable,
            "secrets.lua has Lua that dam cannot edit",
        ),
        (
            Uneditable::ReturnsBuiltInCode,
            "secrets.lua returns a table built in code",
        ),
        (
            Uneditable::TableBuiltInCode,
            "the token table in secrets.lua is built in code",
        ),
    ] {
        assert_eq!(
            text(cannot_edit(why)),
            format!("config: {place}, {add_by_hand}")
        );
    }
    assert_eq!(
        text(ConfigError::CannotEdit {
            table: EditedTable::Remotes,
            why: Uneditable::TableBuiltInCode,
            by_hand: ByHand::AddRemote {
                entry: "todoist = { url = \"todoist::\" }".into(),
            },
        }),
        "config: the remote table in config.lua is built in code, and dam edits only a table \
         written out in full. Add the remote to its remote table by hand: \
         todoist = { url = \"todoist::\" }"
    );
}

#[test]
fn an_edit_that_would_change_the_wrong_entries_is_named() {
    let (table, entry) = tool();
    assert_eq!(
        text(ConfigError::AlreadyHeld { table, entry }),
        "config: secrets.lua already holds token tool"
    );
    let (table, entry) = tool();
    assert_eq!(
        text(ConfigError::NotHeld { table, entry }),
        "config: secrets.lua holds no token tool"
    );
    let (table, entry) = tool();
    let by_hand = ByHand::RemoveToken {
        name: entry.clone(),
    };
    assert_eq!(
        text(ConfigError::HeldUnderUneditableKey {
            table,
            entry,
            by_hand,
        }),
        "config: secrets.lua holds token tool under a key dam cannot edit. \
         Remove token tool from it by hand."
    );
    for (fault, said) in [
        (EditFault::WouldNotHoldTheEntry, "would not hold token tool"),
        (
            EditFault::WouldStillHoldTheEntry,
            "would still hold token tool",
        ),
        (
            EditFault::WouldNotKeepTheOthers,
            "would not keep every other token",
        ),
    ] {
        let (table, entry) = tool();
        assert_eq!(
            text(ConfigError::EditWouldBreak {
                table,
                entry,
                fault,
            }),
            format!("config: the new secrets.lua {said}")
        );
    }
}
