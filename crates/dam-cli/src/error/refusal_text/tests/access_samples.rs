use dam_application::{Refusal, TokenRefusal};
use dam_domain::{Channel, Permission};

pub(super) fn access_refusals_with_their_text() -> Vec<(Refusal, &'static str)> {
    let name = || "tool".to_string();
    [
        (
            TokenRefusal::Malformed,
            "DAM_TOKEN does not hold a damnit token",
        ),
        (
            TokenRefusal::UnknownVersion("v9".into()),
            "DAM_TOKEN holds a v9 token, which this dam does not read",
        ),
        (
            TokenRefusal::Unknown,
            "DAM_TOKEN holds a token that no token.<name> in secrets.lua lists",
        ),
        (
            TokenRefusal::Disabled {
                name: name(),
                why: "unknown key weekdays in token.tool.grant of secrets.lua".into(),
            },
            "token tool is disabled: unknown key weekdays in token.tool.grant of secrets.lua",
        ),
        (
            TokenRefusal::NotYetValid { name: name() },
            "token tool is not valid yet; see its not_before",
        ),
        (
            TokenRefusal::Expired { name: name() },
            "token tool has expired",
        ),
        (
            TokenRefusal::ChannelNotAllowed {
                name: name(),
                channel: Channel::Local,
            },
            "token tool may not be used locally; its via leaves it out",
        ),
        (
            TokenRefusal::MissingDetails {
                name: "claude".into(),
                details: vec!["session".into(), "model".into()],
            },
            "token claude requires the request details session, model; pass each as --meta session=<value> --meta model=<value>",
        ),
    ]
    .into_iter()
    .map(|(refusal, text)| (Refusal::Token(refusal), text))
    .chain([
        (
            Refusal::NotPermitted(Permission::RemotePush),
            "the token does not allow remote:push",
        ),
        (
            Refusal::NotOpenToTokens("remote add"),
            "a token cannot run dam remote add",
        ),
    ])
    .collect()
}

pub(super) fn token_command_refusals_with_their_text() -> Vec<(Refusal, &'static str)> {
    vec![
        (
            Refusal::TokenExists("tool".into()),
            "token tool already exists",
        ),
        (
            Refusal::NotGrantable("config:change".into()),
            "config:change is reserved and cannot be granted",
        ),
        (
            Refusal::NoSuchToken("tool".into()),
            "no token named \"tool\"",
        ),
    ]
}
