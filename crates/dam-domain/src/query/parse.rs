use super::Expr;
use super::term::term;
use crate::Categories;
use lex::{Tok, lex};

pub use error::QueryError;

const MAX_RECURSIVE_DESCENT_DEPTH: usize = 64;

struct Parser<'c> {
    toks: Vec<Tok>,
    at: usize,
    depth: usize,
    categories: &'c Categories,
}

pub fn parse(text: &str, categories: &Categories) -> Result<Expr, QueryError> {
    let toks = lex(text);
    if toks.is_empty() {
        return Err(QueryError::Empty);
    }
    let mut p = Parser {
        toks,
        at: 0,
        depth: 0,
        categories,
    };
    let expr = p.or()?;
    match p.peek() {
        None => Ok(expr),
        Some(Tok::Close) => Err(QueryError::Unexpected(")".into())),
        Some(t) => Err(QueryError::MissingOperator(t.to_string())),
    }
}

impl Parser<'_> {
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
            return self.descend_one_level(|p| Ok(Expr::Not(Box::new(p.not()?))));
        }
        self.atom()
    }

    fn descend_one_level(
        &mut self,
        inner: impl FnOnce(&mut Parser) -> Result<Expr, QueryError>,
    ) -> Result<Expr, QueryError> {
        if self.depth == MAX_RECURSIVE_DESCENT_DEPTH {
            return Err(QueryError::TooDeep);
        }
        self.depth += 1;
        let out = inner(self);
        self.depth -= 1;
        out
    }

    fn atom(&mut self) -> Result<Expr, QueryError> {
        match self.next() {
            Some(Tok::Open) => self.descend_one_level(|p| {
                let inner = p.or()?;
                match p.next() {
                    Some(Tok::Close) => Ok(inner),
                    _ => Err(QueryError::Unclosed),
                }
            }),
            Some(Tok::Word(w)) => term(&w, self.categories).map(Expr::Term),
            Some(other) => Err(QueryError::Unexpected(other.to_string())),
            None => Err(QueryError::EndsEarly),
        }
    }
}

mod error;
mod lex;
