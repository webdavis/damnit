use std::fmt;

use super::QueryError;

#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) enum Tok {
    And,
    Or,
    Not,
    Open,
    Close,
    Word(String),
}

impl fmt::Display for Tok {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Tok::And => f.write_str("&"),
            Tok::Or => f.write_str("|"),
            Tok::Not => f.write_str("!"),
            Tok::Open => f.write_str("("),
            Tok::Close => f.write_str(")"),
            Tok::Word(w) => f.write_str(w),
        }
    }
}

pub(super) fn lex(text: &str) -> Result<Vec<Tok>, QueryError> {
    let mut out = Vec::new();
    let mut word = String::new();
    let flush = |word: &mut String, out: &mut Vec<Tok>| {
        if !word.is_empty() {
            out.push(Tok::Word(std::mem::take(word)));
        }
    };
    let mut chars = text.char_indices();
    while let Some((at, c)) = chars.next() {
        match c {
            '"' => quoted(&mut chars, &mut word)
                .ok_or_else(|| QueryError::UnclosedQuote(text[at..].to_string()))?,
            '&' => {
                flush(&mut word, &mut out);
                out.push(Tok::And)
            }
            '|' => {
                flush(&mut word, &mut out);
                out.push(Tok::Or)
            }
            '!' => {
                flush(&mut word, &mut out);
                out.push(Tok::Not)
            }
            '(' => {
                flush(&mut word, &mut out);
                out.push(Tok::Open)
            }
            ')' => {
                flush(&mut word, &mut out);
                out.push(Tok::Close)
            }
            c if c.is_whitespace() => flush(&mut word, &mut out),
            c => word.push(c),
        }
    }
    flush(&mut word, &mut out);
    Ok(out)
}

fn quoted(chars: &mut std::str::CharIndices, word: &mut String) -> Option<()> {
    loop {
        match chars.next()?.1 {
            '"' => return Some(()),
            '\\' => word.push(chars.next()?.1),
            c => word.push(c),
        }
    }
}
