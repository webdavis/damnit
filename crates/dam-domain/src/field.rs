use std::fmt;

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

impl Field {
    pub fn as_str(&self) -> &'static str {
        match self {
            Field::Subject => "subject",
            Field::Body => "body",
            Field::Path => "path",
            Field::Labels => "labels",
            Field::Depends => "depends",
            Field::Reminders => "reminders",
            Field::Recurrence => "recurrence",
            Field::Priority => "priority",
            Field::Done => "done",
            Field::Due => "due",
            Field::Deadline => "deadline",
            Field::Event => "event",
            Field::Start => "start",
            Field::End => "end",
            Field::Timezone => "timezone",
            Field::Location => "location",
            Field::Attendees => "attendees",
            Field::Status => "status",
            Field::Transparency => "transparency",
            Field::Visibility => "visibility",
            Field::EventType => "event_type",
            Field::Color => "color",
            Field::Organizer => "organizer",
            Field::Conference => "conference",
            Field::Attachments => "attachments",
            Field::Kind => "kind",
        }
    }

    pub fn parse(name: &str) -> Option<Field> {
        Some(match name {
            "subject" => Field::Subject,
            "body" => Field::Body,
            "path" => Field::Path,
            "labels" => Field::Labels,
            "depends" => Field::Depends,
            "reminders" => Field::Reminders,
            "recurrence" => Field::Recurrence,
            "priority" => Field::Priority,
            "done" => Field::Done,
            "due" => Field::Due,
            "deadline" => Field::Deadline,
            "event" => Field::Event,
            "start" => Field::Start,
            "end" => Field::End,
            "timezone" => Field::Timezone,
            "location" => Field::Location,
            "attendees" => Field::Attendees,
            "status" => Field::Status,
            "transparency" => Field::Transparency,
            "visibility" => Field::Visibility,
            "event_type" => Field::EventType,
            "color" => Field::Color,
            "organizer" => Field::Organizer,
            "conference" => Field::Conference,
            "attachments" => Field::Attachments,
            "kind" => Field::Kind,
            _ => return None,
        })
    }
}

impl fmt::Display for Field {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const ALL: [Field; 26] = [
        Field::Subject,
        Field::Body,
        Field::Path,
        Field::Labels,
        Field::Depends,
        Field::Reminders,
        Field::Recurrence,
        Field::Priority,
        Field::Done,
        Field::Due,
        Field::Deadline,
        Field::Event,
        Field::Start,
        Field::End,
        Field::Timezone,
        Field::Location,
        Field::Attendees,
        Field::Status,
        Field::Transparency,
        Field::Visibility,
        Field::EventType,
        Field::Color,
        Field::Organizer,
        Field::Conference,
        Field::Attachments,
        Field::Kind,
    ];

    #[test]
    fn every_field_round_trips_through_its_name() {
        for field in ALL {
            assert_eq!(Field::parse(field.as_str()), Some(field), "{field}");
        }
    }

    #[test]
    fn every_name_is_distinct() {
        let mut names: Vec<&str> = ALL.iter().map(|f| f.as_str()).collect();
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
