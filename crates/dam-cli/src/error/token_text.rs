use std::fmt;

use dam_application::TokenRefusal;
use dam_domain::Channel;

pub(super) struct TokenText<'a>(pub(super) &'a TokenRefusal);

impl fmt::Display for TokenText<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.0 {
            TokenRefusal::Malformed => f.write_str("DAM_TOKEN does not hold a damnit token"),
            TokenRefusal::UnknownVersion(version) => write!(
                f,
                "DAM_TOKEN holds a {version} token, which this dam does not read"
            ),
            TokenRefusal::Unknown => {
                f.write_str("DAM_TOKEN holds a token that no [token.<name>] in the config lists")
            }
            TokenRefusal::Disabled { name, why } => write!(f, "token {name} is disabled: {why}"),
            TokenRefusal::NotYetValid { name } => {
                write!(f, "token {name} is not valid yet; see its not_before")
            }
            TokenRefusal::Expired { name } => write!(f, "token {name} has expired"),
            TokenRefusal::ChannelNotAllowed { name, channel } => write!(
                f,
                "token {name} may not be used {}; its via leaves it out",
                match channel {
                    Channel::Local => "locally",
                    Channel::Mcp => "over MCP",
                }
            ),
            TokenRefusal::MissingDetails { name, details } => write!(
                f,
                "token {name} requires the request details {}, which this dam cannot take yet",
                details.join(", ")
            ),
        }
    }
}
