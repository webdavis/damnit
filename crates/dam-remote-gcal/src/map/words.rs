pub(super) fn response(google: Option<&str>) -> &'static str {
    match google {
        Some("accepted") => "accepted",
        Some("declined") => "declined",
        Some("tentative") => "tentative",
        _ => "needs_action",
    }
}

pub(super) fn visibility(google: Option<&str>) -> &'static str {
    match google {
        Some("public") => "public",
        Some("private") => "private",
        Some("confidential") => "confidential",
        _ => "default",
    }
}

pub(super) fn event_type(google: Option<&str>) -> &'static str {
    match google {
        Some("focusTime") => "focus_time",
        Some("outOfOffice") => "out_of_office",
        Some("workingLocation") => "working_location",
        Some("birthday") => "birthday",
        _ => "default",
    }
}

pub(super) fn status(google: Option<&str>) -> &'static str {
    match google {
        Some("tentative") => "tentative",
        _ => "confirmed",
    }
}

pub(super) fn transparency(google: Option<&str>) -> &'static str {
    match google {
        Some("transparent") => "free",
        _ => "busy",
    }
}

pub(super) fn path_segment(summary: Option<&str>, calendar: &str) -> String {
    let one_segment = |s: &str| s.replace('/', "-");
    match summary.map(one_segment) {
        Some(s) if !s.trim().is_empty() && s != "." && s != ".." => format!("{s}/"),
        _ => format!("{}/", one_segment(calendar)),
    }
}
