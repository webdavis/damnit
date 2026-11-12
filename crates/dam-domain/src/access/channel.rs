use crate::vocabulary::vocabulary;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum Channel {
    Local,
    Mcp,
}

vocabulary!(Channel {
    Local => "local",
    Mcp => "mcp",
});
