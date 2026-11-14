use std::fmt;

mod rules;

use crate::vocabulary::vocabulary;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum Field {
    Subject,
    Body,
    Path,
    Labels,
    Depends,
    Reminders,
    Recurrence,
    Priority,
    Done,
    Due,
    Deadline,
    Attached,
    Estimate,
    Focus,
    Start,
    End,
    Timezone,
    Location,
    Attendees,
    Status,
    Transparency,
    Visibility,
    EventType,
    Color,
    Organizer,
    Conference,
    Attachments,
    Kind,
}

vocabulary!(Field {
    Subject => "subject",
    Body => "body",
    Path => "path",
    Labels => "labels",
    Depends => "depends",
    Reminders => "reminders",
    Recurrence => "recurrence",
    Priority => "priority",
    Done => "done",
    Due => "due",
    Deadline => "deadline",
    Attached => "attached",
    Estimate => "estimate",
    Focus => "focus",
    Start => "start",
    End => "end",
    Timezone => "timezone",
    Location => "location",
    Attendees => "attendees",
    Status => "status",
    Transparency => "transparency",
    Visibility => "visibility",
    EventType => "event_type",
    Color => "color",
    Organizer => "organizer",
    Conference => "conference",
    Attachments => "attachments",
    Kind => "kind",
});

impl fmt::Display for Field {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

#[cfg(test)]
mod tests;
