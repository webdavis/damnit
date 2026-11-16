use super::super::{DateSel, EstimateSel, Term};

pub(super) fn term_texts(term: &Term) -> Vec<(&'static str, String)> {
    match term {
        Term::Done => vec![("done", "true".into()), ("kind", "task".into())],
        term => vec![one(term)],
    }
}

pub(super) fn negated_text(term: &Term) -> Option<(&'static str, String)> {
    match term {
        Term::Done => Some(("done", "false".into())),
        Term::Overdue => None,
        Term::Due(DateSel::None)
        | Term::Deadline(DateSel::None)
        | Term::Focus(DateSel::None)
        | Term::Completed(DateSel::None)
        | Term::Start(DateSel::None)
        | Term::Estimate(EstimateSel::None) => None,
        term => {
            let (key, text) = one(term);
            Some((key, format!("not:{text}")))
        }
    }
}

fn one(term: &Term) -> (&'static str, String) {
    match term {
        Term::Done => ("done", "true".into()),
        Term::Overdue => ("overdue", "true".into()),
        Term::Kind(kind) => ("kind", kind.as_str().into()),
        Term::Due(sel) => ("due", date(sel)),
        Term::Deadline(sel) => ("deadline", date(sel)),
        Term::Focus(sel) => ("focus.day", date(sel)),
        Term::Completed(sel) => ("completed_at", date(sel)),
        Term::Start(sel) => ("start", date(sel)),
        Term::Estimate(sel) => ("estimate", estimate(*sel)),
        Term::Path(path) => ("path", exact(path.as_str())),
        Term::Label(label) => ("labels", exact(label)),
        Term::Priority(p) => ("priority", p.get().to_string()),
        Term::Transparency(t) => ("transparency", t.as_str().into()),
        Term::Attached(oid) => ("attached", oid.to_string()),
        Term::Subject(text) => ("subject", exact(text)),
    }
}

fn date(sel: &DateSel) -> String {
    match sel {
        DateSel::Today => "today".into(),
        DateSel::Tomorrow => "today+1".into(),
        DateSel::Yesterday => "today-1".into(),
        DateSel::On(day) => day.to_string(),
        DateSel::ThisWeek => "week".into(),
        DateSel::NextWeek => "week+1".into(),
        DateSel::Before(day) => day
            .yesterday()
            .map_or_else(|_| format!("..{day}"), |before| format!("..{before}")),
        DateSel::After(day) => day
            .tomorrow()
            .map_or_else(|_| format!("{day}.."), |after| format!("{after}..")),
        DateSel::None => "none".into(),
    }
}

fn estimate(sel: EstimateSel) -> String {
    match sel {
        EstimateSel::Max(minutes) => format!("..{minutes}m"),
        EstimateSel::Min(minutes) => format!("{minutes}m.."),
        EstimateSel::None => "none".into(),
    }
}

fn exact(text: &str) -> String {
    let plain = !text.contains(',')
        && !text.starts_with("not:")
        && !text.starts_with('=')
        && text != "none";
    match plain {
        true => text.to_string(),
        false => format!("={text}"),
    }
}
