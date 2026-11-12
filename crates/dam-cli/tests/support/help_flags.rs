use super::sandbox::Sandbox;

fn flags(sb: &Sandbox, verb: &[&str]) -> Vec<String> {
    let (ok, out, err) = sb.dam(&[verb, &["--help"]].concat());
    assert!(ok, "{err}");
    out.lines()
        .filter_map(|line| {
            let mut words = line.split_whitespace();
            let first = words.next()?;
            let flag = if first.starts_with("--") {
                first
            } else {
                words.next()?
            };
            flag.strip_prefix("--").filter(|_| first.starts_with('-'))
        })
        .map(|flag| flag.replace('-', "_"))
        .collect()
}

pub fn own_flags(sb: &Sandbox, verb: &[&str]) -> Vec<String> {
    let global = flags(sb, &["log"]);
    flags(sb, verb)
        .into_iter()
        .filter(|flag| !global.contains(flag))
        .collect()
}
