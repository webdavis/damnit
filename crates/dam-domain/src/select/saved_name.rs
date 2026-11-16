use super::on_day::OnDay;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum NameFault {
    Empty,
    None,
    OnDay,
    Starts(&'static str),
    Comma,
}

const STARTS: [&str; 4] = ["not:", "=", "in:", "on:"];

pub fn filter_name_fault(name: &str) -> Option<NameFault> {
    if name.is_empty() {
        return Some(NameFault::Empty);
    }
    if name == "none" {
        return Some(NameFault::None);
    }
    if let Some(start) = STARTS.iter().find(|s| name.starts_with(**s)) {
        return Some(NameFault::Starts(start));
    }
    if name.contains(',') {
        return Some(NameFault::Comma);
    }
    match OnDay::read(name) {
        Some(OnDay::EventsOf(_)) | None => None,
        Some(_) => Some(NameFault::OnDay),
    }
}
