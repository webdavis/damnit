pub fn calendars(address: &str) -> Vec<String> {
    let mut out: Vec<String> = Vec::new();
    for id in address
        .split(',')
        .map(str::trim)
        .filter(|id| !id.is_empty())
    {
        if !out.iter().any(|seen| seen == id) {
            out.push(id.to_string());
        }
    }
    if out.is_empty() {
        out.push("primary".into());
    }
    out
}

#[cfg(test)]
mod tests;
