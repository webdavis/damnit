#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) enum Tok {
    And,
    Or,
    Not,
    Open,
    Close,
    Word(String),
}

pub(super) fn lex(text: &str) -> Option<Vec<Tok>> {
    let mut out = Vec::new();
    let mut word = String::new();
    let mut chars = text.chars();
    while let Some(c) = chars.next() {
        let operator = match c {
            '&' => Some(Tok::And),
            '|' => Some(Tok::Or),
            '!' => Some(Tok::Not),
            '(' => Some(Tok::Open),
            ')' => Some(Tok::Close),
            _ => None,
        };
        if let Some(operator) = operator {
            flush(&mut word, &mut out);
            out.push(operator);
        } else if c.is_whitespace() {
            flush(&mut word, &mut out);
        } else if c == '"' {
            quoted(&mut chars, &mut word)?;
        } else {
            word.push(c);
        }
    }
    flush(&mut word, &mut out);
    Some(out)
}

fn flush(word: &mut String, out: &mut Vec<Tok>) {
    if !word.is_empty() {
        out.push(Tok::Word(std::mem::take(word)));
    }
}

fn quoted(chars: &mut std::str::Chars, word: &mut String) -> Option<()> {
    loop {
        match chars.next()? {
            '"' => return Some(()),
            '\\' => word.push(chars.next()?),
            c => word.push(c),
        }
    }
}
