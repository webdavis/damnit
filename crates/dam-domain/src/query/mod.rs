mod parse;
mod table;
mod term;

use crate::{Date, Kind, Oid, Path, Priority, Transparency};

use parse::parse;
pub use table::query_table;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum DateSel {
    Today,
    Tomorrow,
    Yesterday,
    On(Date),
    ThisWeek,
    NextWeek,
    Before(Date),
    After(Date),
    None,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum EstimateSel {
    Max(u32),
    Min(u32),
    None,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Term {
    Done,
    Overdue,
    Kind(Kind),
    Due(DateSel),
    Deadline(DateSel),
    Estimate(EstimateSel),
    Focus(DateSel),
    Completed(DateSel),
    Start(DateSel),
    Path(Path),
    Label(String),
    Priority(Priority),
    Transparency(Transparency),
    Attached(Oid),
    Subject(String),
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Expr {
    And(Box<Expr>, Box<Expr>),
    Or(Box<Expr>, Box<Expr>),
    Not(Box<Expr>),
    Term(Term),
}
