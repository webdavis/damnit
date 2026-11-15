use super::sandbox::Sandbox;

fn section(sb: &Sandbox, command: &[&str], title: &str) -> Vec<String> {
    let (ok, out, err) = sb.dam(&[command, &["--help"]].concat());
    assert!(ok, "{err}");
    out.lines()
        .skip_while(|line| *line != title)
        .skip(1)
        .take_while(|line| !line.is_empty())
        .filter_map(|line| line.strip_prefix("  "))
        .filter(|entry| !entry.starts_with(' '))
        .filter_map(|entry| entry.split_whitespace().next().map(str::to_string))
        .collect()
}

pub fn every_command(sb: &Sandbox) -> Vec<Vec<String>> {
    let mut found = Vec::new();
    walk(sb, &[], &mut found);
    found
}

fn walk(sb: &Sandbox, path: &[String], found: &mut Vec<Vec<String>>) {
    let words: Vec<&str> = path.iter().map(String::as_str).collect();
    let subcommands: Vec<String> = section(sb, &words, "Commands:")
        .into_iter()
        .filter(|name| name != "help")
        .collect();
    if subcommands.is_empty() {
        found.push(path.to_vec());
    }
    for name in subcommands {
        walk(sb, &[path, &[name]].concat(), found);
    }
}

pub fn arguments(sb: &Sandbox, command: &[&str]) -> Vec<String> {
    section(sb, command, "Arguments:")
        .into_iter()
        .map(|word| word.trim_matches(['<', '>', '[', ']', '.']).to_lowercase())
        .collect()
}
