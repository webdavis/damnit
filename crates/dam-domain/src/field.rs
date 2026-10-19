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
    Event,
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
    Event => "event",
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
mod tests {
    use super::*;

    #[test]
    fn every_field_round_trips_through_its_name() {
        for field in Field::ALL {
            assert_eq!(Field::parse(field.as_str()), Some(field), "{field}");
        }
    }

    #[test]
    fn every_name_is_distinct() {
        let mut names: Vec<&str> = Field::ALL.iter().map(|f| f.as_str()).collect();
        names.sort_unstable();
        let count = names.len();
        names.dedup();
        assert_eq!(names.len(), count, "two fields share a name");
    }

    #[test]
    fn an_unknown_name_is_none_so_a_field_from_a_newer_protocol_is_ignored_not_refused() {
        assert_eq!(Field::parse("eventtype"), None);
        assert_eq!(Field::parse(""), None);
    }
}
