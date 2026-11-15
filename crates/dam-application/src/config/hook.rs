pub const STARTED_BY_HOOK_VARIABLE: &str = "DAM_HOOK";

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum HookEvent {
    TaskNew,
    TaskChanged,
    TaskDone,
    TaskPulled,
}

impl HookEvent {
    pub const LOCAL: [HookEvent; 3] = [
        HookEvent::TaskNew,
        HookEvent::TaskChanged,
        HookEvent::TaskDone,
    ];

    pub const fn as_str(self) -> &'static str {
        match self {
            HookEvent::TaskNew => "task_new",
            HookEvent::TaskChanged => "task_changed",
            HookEvent::TaskDone => "task_done",
            HookEvent::TaskPulled => "task_pulled",
        }
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum OnFailure {
    #[default]
    Warn,
    Refuse,
}

impl OnFailure {
    pub const ALL: [OnFailure; 2] = [OnFailure::Warn, OnFailure::Refuse];

    pub const fn as_str(self) -> &'static str {
        match self {
            OnFailure::Warn => "warn",
            OnFailure::Refuse => "refuse",
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct HookConfig {
    pub event: HookEvent,
    pub on_failure: Option<OnFailure>,
}

impl HookConfig {
    pub fn on_failure(&self) -> OnFailure {
        self.on_failure.unwrap_or_default()
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum PulledChanges {
    #[default]
    Committed,
    Working,
}

impl PulledChanges {
    pub const ALL: [PulledChanges; 2] = [PulledChanges::Committed, PulledChanges::Working];

    pub const fn as_str(self) -> &'static str {
        match self {
            PulledChanges::Committed => "committed",
            PulledChanges::Working => "working",
        }
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct PulledHook {
    pub changes: Option<PulledChanges>,
}

impl PulledHook {
    pub fn changes(&self) -> PulledChanges {
        self.changes.unwrap_or_default()
    }
}
