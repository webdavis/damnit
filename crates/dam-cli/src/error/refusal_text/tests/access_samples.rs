use dam_application::{Refusal, TokenRefusal};
use dam_domain::{Channel, Permission};

pub(super) fn access_refusals_with_their_text() -> Vec<(Refusal, &'static str)> {
    let name = || "vpt".to_string();
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
            "DAM_TOKEN holds a token that no [token.<name>] in the config lists",
        ),
        (
            TokenRefusal::Disabled {
                name: name(),
                why: "unknown key weekdays in [token.vpt.grant]".into(),
            },
            "token vpt is disabled: unknown key weekdays in [token.vpt.grant]",
        ),
        (
            TokenRefusal::NotYetValid { name: name() },
            "token vpt is not valid yet; see its not_before",
        ),
        (
            TokenRefusal::Expired { name: name() },
            "token vpt has expired",
        ),
        (
            TokenRefusal::ChannelNotAllowed {
                name: name(),
                channel: Channel::Local,
            },
            "token vpt may not be used locally; its via leaves it out",
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
