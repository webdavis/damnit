use super::Expr;
use super::term::term;
use crate::Categories;
use lex::{Tok, lex};

const MAX_RECURSIVE_DESCENT_DEPTH: usize = 64;

struct Parser<'c> {
    toks: Vec<Tok>,
    at: usize,
    depth: usize,
    categories: &'c Categories,
}

pub(super) fn parse(text: &str, categories: &Categories) -> Option<Expr> {
    let mut p = Parser {
        toks: lex(text)?,
        at: 0,
        depth: 0,
        categories,
    };
    let expr = p.or()?;
    p.peek().is_none().then_some(expr)
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

    fn or(&mut self) -> Option<Expr> {
        let mut left = self.and()?;
        while self.peek() == Some(&Tok::Or) {
            self.next();
            let right = self.and()?;
            left = Expr::Or(Box::new(left), Box::new(right));
        }
        Some(left)
    }

    fn and(&mut self) -> Option<Expr> {
        let mut left = self.not()?;
        while self.peek() == Some(&Tok::And) {
            self.next();
            let right = self.not()?;
            left = Expr::And(Box::new(left), Box::new(right));
        }
        Some(left)
    }

    fn not(&mut self) -> Option<Expr> {
        if self.peek() == Some(&Tok::Not) {
            self.next();
            return self.descend_one_level(|p| Some(Expr::Not(Box::new(p.not()?))));
        }
        self.atom()
    }

    fn descend_one_level(
        &mut self,
        inner: impl FnOnce(&mut Parser) -> Option<Expr>,
    ) -> Option<Expr> {
        if self.depth == MAX_RECURSIVE_DESCENT_DEPTH {
            return None;
        }
        self.depth += 1;
        let out = inner(self);
        self.depth -= 1;
        out
    }

    fn atom(&mut self) -> Option<Expr> {
        match self.next()? {
            Tok::Open => self.descend_one_level(|p| {
                let inner = p.or()?;
                (p.next()? == Tok::Close).then_some(inner)
            }),
            Tok::Word(w) => term(&w, self.categories).map(Expr::Term),
            _ => None,
        }
    }
}

mod lex;
