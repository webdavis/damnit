use crate::vocabulary::vocabulary;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum EventStatus {
    Confirmed,
    Tentative,
    Cancelled,
}

vocabulary!(EventStatus {
    Confirmed => "confirmed",
    Tentative => "tentative",
    Cancelled => "cancelled",
});

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Transparency {
    Busy,
    Free,
}

vocabulary!(Transparency {
    Busy => "busy",
    Free => "free",
});

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Visibility {
    Default,
    Public,
    Private,
    Confidential,
}

vocabulary!(Visibility {
    Default => "default",
    Public => "public",
    Private => "private",
    Confidential => "confidential",
});

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum EventType {
    Default,
    FocusTime,
    OutOfOffice,
    WorkingLocation,
    Birthday,
}

vocabulary!(EventType {
    Default => "default",
    FocusTime => "focus_time",
    OutOfOffice => "out_of_office",
    WorkingLocation => "working_location",
    Birthday => "birthday",
});

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum ResponseStatus {
    NeedsAction,
    Accepted,
    Declined,
    Tentative,
}

vocabulary!(ResponseStatus {
    NeedsAction => "needs_action",
    Accepted => "accepted",
    Declined => "declined",
    Tentative => "tentative",
});
