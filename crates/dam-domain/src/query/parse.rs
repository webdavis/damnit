use std::fmt;

use super::Expr;
use super::term::term;

#[derive(Debug, PartialEq, Eq)]
pub enum QueryError {
    Empty,
    Unexpected(String),
    BadValue {
        key: String,
        value: String,
    },
    Unclosed,
    /// More nesting than `MAX_DEPTH`, refused rather than recursed into.
    TooDeep,
}

#[derive(Clone, Debug, PartialEq, Eq)]
enum Tok {
    And,
    Or,
    Not,
    Open,
    Close,
    Word(String),
}

fn lex(text: &str) -> Vec<Tok> {
    let mut out = Vec::new();
    let mut word = String::new();
    let flush = |word: &mut String, out: &mut Vec<Tok>| {
        if !word.is_empty() {
            out.push(Tok::Word(std::mem::take(word)));
        }
    };
    for c in text.chars() {
        match c {
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
    out
}

/// How deep `(` and `!` may nest. The parser descends once per level, so an
/// unbounded query would overflow the stack and abort the process instead of
/// answering with an error.
const MAX_DEPTH: usize = 64;

struct Parser {
    toks: Vec<Tok>,
    at: usize,
    depth: usize,
}

pub fn parse(text: &str) -> Result<Expr, QueryError> {
    let toks = lex(text);
    if toks.is_empty() {
        return Err(QueryError::Empty);
    }
    let mut p = Parser {
        toks,
        at: 0,
        depth: 0,
    };
    let expr = p.or()?;
    match p.peek() {
        None => Ok(expr),
        Some(Tok::Close) => Err(QueryError::Unexpected(")".into())),
        Some(t) => Err(QueryError::Unexpected(format!("{t:?}"))),
    }
}

impl Parser {
    fn peek(&self) -> Option<&Tok> {
        self.toks.get(self.at)
    }

    fn next(&mut self) -> Option<Tok> {
        let t = self.toks.get(self.at).cloned();
        self.at += 1;
        t
    }

    fn or(&mut self) -> Result<Expr, QueryError> {
        let mut left = self.and()?;
        while self.peek() == Some(&Tok::Or) {
            self.next();
            let right = self.and()?;
            left = Expr::Or(Box::new(left), Box::new(right));
        }
        Ok(left)
    }

    fn and(&mut self) -> Result<Expr, QueryError> {
        let mut left = self.not()?;
        while self.peek() == Some(&Tok::And) {
            self.next();
            let right = self.not()?;
            left = Expr::And(Box::new(left), Box::new(right));
        }
        Ok(left)
    }

    fn not(&mut self) -> Result<Expr, QueryError> {
        if self.peek() == Some(&Tok::Not) {
            self.next();
            return self.deeper(|p| Ok(Expr::Not(Box::new(p.not()?))));
        }
        self.atom()
    }

    /// Runs one level of descent, refusing to go past `MAX_DEPTH`.
    fn deeper(
        &mut self,
        inner: impl FnOnce(&mut Parser) -> Result<Expr, QueryError>,
    ) -> Result<Expr, QueryError> {
        if self.depth == MAX_DEPTH {
            return Err(QueryError::TooDeep);
        }
        self.depth += 1;
        let out = inner(self);
        self.depth -= 1;
        out
    }

    fn atom(&mut self) -> Result<Expr, QueryError> {
        match self.next() {
            Some(Tok::Open) => self.deeper(|p| {
                let inner = p.or()?;
                match p.next() {
                    Some(Tok::Close) => Ok(inner),
                    _ => Err(QueryError::Unclosed),
                }
            }),
            Some(Tok::Word(w)) => term(&w).map(Expr::Term),
            Some(Tok::Close) => Err(QueryError::Unexpected(")".into())),
            Some(other) => Err(QueryError::Unexpected(format!("{other:?}"))),
            None => Err(QueryError::Unexpected("end".into())),
        }
    }
}

impl fmt::Display for QueryError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            QueryError::Empty => f.write_str("the query is empty"),
            QueryError::Unexpected(t) => write!(f, "unexpected {t} in query"),
            QueryError::BadValue { key, value } => write!(f, "{key}: cannot read {value:?}"),
            QueryError::Unclosed => f.write_str("a ( has no matching )"),
            QueryError::TooDeep => write!(f, "the query nests deeper than {MAX_DEPTH}"),
        }
    }
}

impl std::error::Error for QueryError {}
