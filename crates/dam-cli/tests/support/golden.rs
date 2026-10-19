use std::path::PathBuf;

use super::sandbox::Sandbox;

pub fn assert_golden(name: &str, actual: &str) {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests/golden")
        .join(name);
    if std::env::var_os("GOLDEN_BLESS").is_some() {
        std::fs::write(&path, actual).unwrap();
        return;
    }
    let expected = std::fs::read_to_string(&path)
        .unwrap_or_else(|e| panic!("{}: {e}; GOLDEN_BLESS=1 writes it", path.display()));
    assert_eq!(
        actual, expected,
        "dam's output drifted from the golden {name}"
    );
}

pub fn assert_golden_output(sb: &Sandbox, args: &[&str], name: &str) {
    let (ok, out, err) = sb.dam(args);
    assert!(ok, "{args:?}: {err}");
    assert_golden(name, &normalize(&out, &super::fixture::seeded_oids()));
}

pub fn normalize(text: &str, seeded: &[String]) -> String {
    let bytes = text.as_bytes();
    let mut ids: Vec<String> = Vec::new();
    let mut out = String::new();
    let mut i = 0;
    while i < bytes.len() {
        let at_word_start = i == 0 || !bytes[i - 1].is_ascii_alphanumeric();
        if at_word_start {
            if let Some(len) = clock_time_len(&text[i..]) {
                out.push_str("<now>");
                i += len;
                continue;
            }
            let run = hex_word_len(&bytes[i..]);
            let word = &text[i..i + run];
            if (run == 40 || run == 7) && !seeded.iter().any(|s| s.starts_with(word)) {
                out.push_str(&placeholder(&mut ids, word));
                i += run;
                continue;
            }
        }
        let ch = text[i..].chars().next().unwrap();
        out.push(ch);
        i += ch.len_utf8();
    }
    out
}

fn hex_word_len(bytes: &[u8]) -> usize {
    let run = bytes
        .iter()
        .take_while(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(b))
        .count();
    match bytes.get(run) {
        Some(b) if b.is_ascii_alphanumeric() => 0,
        _ => run,
    }
}

fn placeholder(ids: &mut Vec<String>, word: &str) -> String {
    let n = match ids
        .iter()
        .position(|id| id.starts_with(word) || word.starts_with(id.as_str()))
    {
        Some(n) => n,
        None => {
            ids.push(word.to_string());
            ids.len() - 1
        }
    };
    format!("<id-{}>", n + 1)
}

fn clock_time_len(s: &str) -> Option<usize> {
    let date: jiff::civil::Date = s.get(..10)?.parse().ok()?;
    let rest = &s[10..];
    if let Some(time) = rest.strip_prefix('T') {
        let digits = time
            .bytes()
            .take_while(|b| b"0123456789:.".contains(b))
            .count();
        return (time.as_bytes().get(digits) == Some(&b'Z')).then_some(10 + 1 + digits + 1);
    }
    let today = jiff::Timestamp::now()
        .to_zoned(jiff::tz::TimeZone::UTC)
        .date();
    let minute = rest.get(..6)?;
    let is_clock = minute.starts_with(' ') && minute[1..].parse::<jiff::civil::Time>().is_ok();
    (is_clock && (date == today || date == today.yesterday().ok()?)).then_some(16)
}
