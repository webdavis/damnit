use std::fmt;

use super::{DateSel, Expr, Term};
use crate::{Kind, Oid, Path, Priority, Transparency};

#[derive(Debug, PartialEq, Eq)]
pub enum QueryError {
    Empty,
    Unexpected(String),
    BadValue { key: String, value: String },
    Unclosed,
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

struct Parser {
    toks: Vec<Tok>,
    at: usize,
}

pub fn parse(text: &str) -> Result<Expr, QueryError> {
    let toks = lex(text);
    if toks.is_empty() {
        return Err(QueryError::Empty);
    }
    let mut p = Parser { toks, at: 0 };
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
            return Ok(Expr::Not(Box::new(self.not()?)));
        }
        self.atom()
    }

    fn atom(&mut self) -> Result<Expr, QueryError> {
        match self.next() {
            Some(Tok::Open) => {
                let inner = self.or()?;
                match self.next() {
                    Some(Tok::Close) => Ok(inner),
                    _ => Err(QueryError::Unclosed),
                }
            }
            Some(Tok::Word(w)) => term(&w).map(Expr::Term),
            Some(Tok::Close) => Err(QueryError::Unexpected(")".into())),
            Some(other) => Err(QueryError::Unexpected(format!("{other:?}"))),
            None => Err(QueryError::Unexpected("end".into())),
        }
    }
}

fn term(word: &str) -> Result<Term, QueryError> {
    if let Some(label) = word.strip_prefix('@') {
        return Ok(Term::Label(label.to_string()));
    }
    if let Some(n) = word.strip_prefix('p').and_then(|n| n.parse::<u8>().ok()) {
        return Priority::new(n)
            .map(Term::Priority)
            .map_err(|_| bad("priority", word));
    }
    let (key, value) = match word.split_once(':') {
        Some(kv) => kv,
        None => {
            return match word {
                "done" => Ok(Term::Done),
                "overdue" => Ok(Term::Overdue),
                other => Err(QueryError::Unexpected(other.to_string())),
            };
        }
    };
    Ok(match key {
        "kind" => Term::Kind(match value {
            "task" => Kind::Task,
            "event" => Kind::Event,
            _ => return Err(bad(key, value)),
        }),
        "due" => Term::Due(date_sel(key, value)?),
        "deadline" => Term::Deadline(date_sel(key, value)?),
        "start" => Term::Start(date_sel(key, value)?),
        "path" => Term::Path(Path::parse(value).map_err(|_| bad(key, value))?),
        "label" => Term::Label(value.to_string()),
        "priority" => Term::Priority(
            value
                .parse::<u8>()
                .ok()
                .and_then(|n| Priority::new(n).ok())
                .ok_or_else(|| bad(key, value))?,
        ),
        "transparency" => Term::Transparency(match value {
            "busy" => Transparency::Busy,
            "free" => Transparency::Free,
            _ => return Err(bad(key, value)),
        }),
        "attached" => Term::Attached(Oid::parse(value).map_err(|_| bad(key, value))?),
        "subject" => Term::Subject(value.to_string()),
        name => Term::Category {
            name: name.to_string(),
            value: value.to_string(),
        },
    })
}

fn date_sel(key: &str, value: &str) -> Result<DateSel, QueryError> {
    Ok(match value {
        "today" => DateSel::Today,
        "tomorrow" => DateSel::Tomorrow,
        "yesterday" => DateSel::Yesterday,
        "this-week" => DateSel::ThisWeek,
        "next-week" => DateSel::NextWeek,
        "none" => DateSel::None,
        v => {
            if let Some(d) = v.strip_prefix("before:") {
                DateSel::Before(d.parse().map_err(|_| bad(key, value))?)
            } else if let Some(d) = v.strip_prefix("after:") {
                DateSel::After(d.parse().map_err(|_| bad(key, value))?)
            } else {
                DateSel::On(v.parse().map_err(|_| bad(key, value))?)
            }
        }
    })
}

fn bad(key: &str, value: &str) -> QueryError {
    QueryError::BadValue {
        key: key.to_string(),
        value: value.to_string(),
    }
}

impl fmt::Display for QueryError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            QueryError::Empty => f.write_str("the query is empty"),
            QueryError::Unexpected(t) => write!(f, "unexpected {t} in query"),
            QueryError::BadValue { key, value } => write!(f, "{key}: cannot read {value:?}"),
            QueryError::Unclosed => f.write_str("a ( has no matching )"),
        }
    }
}

impl std::error::Error for QueryError {}
