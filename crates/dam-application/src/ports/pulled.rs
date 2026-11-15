use crate::ports::RemoteName;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PulledChange {
    New,
    Changed,
    Done,
}

impl PulledChange {
    pub const fn as_str(self) -> &'static str {
        match self {
            PulledChange::New => "new",
            PulledChange::Changed => "changed",
            PulledChange::Done => "done",
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Pull {
    pub remote: RemoteName,
    pub change: PulledChange,
}
