use super::Field;
use crate::{Completion, Event, Object, Task};

impl Field {
    pub fn same(self, a: &Object, b: &Object) -> bool {
        let (x, y) = (a.base(), b.base());
        match self {
            Field::Subject => x.subject == y.subject,
            Field::Body => x.body == y.body,
            Field::Path => x.path == y.path,
            Field::Labels => x.labels == y.labels,
            Field::Depends => x.depends == y.depends,
            Field::Reminders => x.reminders == y.reminders,
            Field::Recurrence => x.recurrence == y.recurrence,
            Field::Priority => tasks_agree(a, b, |x, y| x.priority == y.priority),
            Field::Done => tasks_agree(a, b, |x, y| x.completion == y.completion),
            Field::Due => tasks_agree(a, b, |x, y| x.due == y.due),
            Field::Deadline => tasks_agree(a, b, |x, y| x.deadline == y.deadline),
            Field::Event => tasks_agree(a, b, |x, y| x.event == y.event),
            Field::Start => events_agree(a, b, |x, y| x.start == y.start),
            Field::End => events_agree(a, b, |x, y| x.end == y.end),
            Field::Timezone => events_agree(a, b, |x, y| x.timezone == y.timezone),
            Field::Location => events_agree(a, b, |x, y| x.location == y.location),
            Field::Attendees => events_agree(a, b, |x, y| x.attendees == y.attendees),
            Field::Status => events_agree(a, b, |x, y| x.status == y.status),
            Field::Transparency => events_agree(a, b, |x, y| x.transparency == y.transparency),
            Field::Visibility => events_agree(a, b, |x, y| x.visibility == y.visibility),
            Field::EventType => events_agree(a, b, |x, y| x.event_type == y.event_type),
            Field::Color => events_agree(a, b, |x, y| x.color == y.color),
            Field::Organizer => events_agree(a, b, |x, y| x.organizer == y.organizer),
            Field::Conference => events_agree(a, b, |x, y| x.conference == y.conference),
            Field::Attachments => events_agree(a, b, |x, y| x.attachments == y.attachments),
            Field::Kind => a.kind() == b.kind(),
        }
    }

    pub fn copy(self, from: &Object, to: &mut Object) {
        let (f, t) = (from.base(), to.base_mut());
        match self {
            Field::Subject => t.subject = f.subject.clone(),
            Field::Body => t.body = f.body.clone(),
            Field::Path => t.path = f.path.clone(),
            Field::Labels => t.labels = f.labels.clone(),
            Field::Depends => t.depends = f.depends.clone(),
            Field::Reminders => t.reminders = f.reminders.clone(),
            Field::Recurrence => t.recurrence = f.recurrence.clone(),
            Field::Priority => between_tasks(from, to, |f, t| t.priority = f.priority),
            Field::Done => between_tasks(from, to, |f, t| {
                t.completion = incoming_completion(&t.completion, &f.completion)
            }),
            Field::Due => between_tasks(from, to, |f, t| t.due = f.due.clone()),
            Field::Deadline => between_tasks(from, to, |f, t| t.deadline = f.deadline),
            Field::Event => between_tasks(from, to, |f, t| t.event = f.event.clone()),
            Field::Start => between_events(from, to, |f, t| t.start = f.start.clone()),
            Field::End => between_events(from, to, |f, t| t.end = f.end.clone()),
            Field::Timezone => between_events(from, to, |f, t| t.timezone = f.timezone.clone()),
            Field::Location => between_events(from, to, |f, t| t.location = f.location.clone()),
            Field::Attendees => between_events(from, to, |f, t| t.attendees = f.attendees.clone()),
            Field::Status => between_events(from, to, |f, t| t.status = f.status),
            Field::Transparency => between_events(from, to, |f, t| t.transparency = f.transparency),
            Field::Visibility => between_events(from, to, |f, t| t.visibility = f.visibility),
            Field::EventType => between_events(from, to, |f, t| t.event_type = f.event_type),
            Field::Color => between_events(from, to, |f, t| t.color = f.color.clone()),
            Field::Organizer => between_events(from, to, |f, t| t.organizer = f.organizer.clone()),
            Field::Conference => {
                between_events(from, to, |f, t| t.conference = f.conference.clone())
            }
            Field::Attachments => {
                between_events(from, to, |f, t| t.attachments = f.attachments.clone())
            }
            Field::Kind => {}
        }
    }
}

fn tasks_agree(a: &Object, b: &Object, same: impl Fn(&Task, &Task) -> bool) -> bool {
    match (a, b) {
        (Object::Task(x), Object::Task(y)) => same(x, y),
        _ => true,
    }
}

fn events_agree(a: &Object, b: &Object, same: impl Fn(&Event, &Event) -> bool) -> bool {
    match (a, b) {
        (Object::Event(x), Object::Event(y)) => same(x, y),
        _ => true,
    }
}

fn between_tasks(from: &Object, to: &mut Object, copy: impl Fn(&Task, &mut Task)) {
    if let (Object::Task(f), Object::Task(t)) = (from, to) {
        copy(f, t)
    }
}

fn between_events(from: &Object, to: &mut Object, copy: impl Fn(&Event, &mut Event)) {
    if let (Object::Event(f), Object::Event(t)) = (from, to) {
        copy(f, t)
    }
}

fn incoming_completion(ours: &Completion, theirs: &Completion) -> Completion {
    match (ours, theirs) {
        (Completion::Done { at: Some(_) }, Completion::Done { at: None }) => ours.clone(),
        _ => theirs.clone(),
    }
}
