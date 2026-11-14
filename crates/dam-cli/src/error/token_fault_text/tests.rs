use dam_application::{Expected, Misread, Place, TokenFault, TokenWarning};

use super::{TokenFaultText, TokenWarningText};

fn via_not_a_list() -> Misread {
    Misread::WrongType {
        key: "via".into(),
        at: Place {
            file: "secrets.lua".into(),
            table: Some("token.tool".into()),
        },
        expected: Expected::ListOfTexts,
    }
}

#[test]
fn each_reason_a_token_is_disabled_reads_as_today() {
    let cases = [
        (
            via_not_a_list().into(),
            "via must be a list of strings in token.tool of secrets.lua",
        ),
        (TokenFault::NoValue, "it has no value"),
        (TokenFault::ValueNotText, "its value must be a string"),
        (
            TokenFault::ValueNotAToken,
            "its value is not a damnit token",
        ),
        (
            TokenFault::LimitsNotEnforced,
            "its limits are not enforced yet",
        ),
        (
            TokenFault::Reserved("config:change".into()),
            "config:change is reserved and cannot be granted",
        ),
        (
            TokenFault::NewerSchema { found: 2, reads: 1 },
            "secrets.lua is schema 2, and this dam reads schema 1",
        ),
        (
            TokenFault::SchemaNotWhole {
                found: "1.5".into(),
            },
            "schema in secrets.lua must be a whole number, and it is 1.5",
        ),
        (
            TokenFault::SharesValue {
                with: vec!["a".into(), "b".into()],
            },
            "it holds the same value as token a, token b",
        ),
    ];
    for (fault, said) in cases {
        assert_eq!(TokenFaultText(&fault).to_string(), said);
    }
}

#[test]
fn each_token_warning_names_the_token() {
    let cases = [
        (
            TokenWarning::EveryTokenIgnored(via_not_a_list()),
            "every token is ignored: via must be a list of strings in token.tool of secrets.lua",
        ),
        (
            TokenWarning::Ignored {
                name: "tool".into(),
                why: TokenFault::NoValue,
            },
            "token tool is ignored: it has no value",
        ),
        (
            TokenWarning::UnknownPermission {
                name: "tool".into(),
                word: "task:fly".into(),
            },
            "token tool: unknown permission \"task:fly\" is skipped",
        ),
        (
            TokenWarning::UnknownChannel {
                name: "tool".into(),
                word: "pigeon".into(),
            },
            "token tool: unknown channel \"pigeon\" is skipped",
        ),
    ];
    for (warning, said) in cases {
        assert_eq!(TokenWarningText(&warning).to_string(), said);
    }
}
