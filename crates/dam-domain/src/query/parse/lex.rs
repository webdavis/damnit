use std::fmt;
use std::ops::Range;

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

pub(super) fn lex(text: &str) -> Result<Vec<(Tok, Range<usize>)>, QueryError> {
    let mut out = Vec::new();
    let mut word = Word::default();
    let mut chars = text.char_indices();
    while let Some((at, c)) = chars.next() {
        let operator = match c {
            '&' => Some(Tok::And),
            '|' => Some(Tok::Or),
            '!' => Some(Tok::Not),
            '(' => Some(Tok::Open),
            ')' => Some(Tok::Close),
            _ => None,
        };
        if let Some(operator) = operator {
            word.flush(&mut out);
            out.push((operator, at..at + 1));
        } else if c.is_whitespace() {
            word.flush(&mut out);
        } else {
            word.start.get_or_insert(at);
            if c == '"' {
                quoted(&mut chars, &mut word.text)
                    .ok_or_else(|| QueryError::UnclosedQuote(text[at..].to_string()))?;
            } else {
                word.text.push(c);
            }
            word.end = chars.offset();
        }
    }
    word.flush(&mut out);
    Ok(out)
}

#[derive(Default)]
struct Word {
    text: String,
    start: Option<usize>,
    end: usize,
}

impl Word {
    fn flush(&mut self, out: &mut Vec<(Tok, Range<usize>)>) {
        if let Some(start) = self.start.take()
            && !self.text.is_empty()
        {
            out.push((Tok::Word(std::mem::take(&mut self.text)), start..self.end));
        }
    }
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
