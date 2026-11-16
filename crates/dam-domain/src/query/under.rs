use super::{Expr, Term};
use crate::Path;

impl Expr {
    pub fn under(self, place: &Path) -> Expr {
        let under = |e: Box<Expr>| Box::new(e.under(place));
        match self {
            Expr::And(a, b) => Expr::And(under(a), under(b)),
            Expr::Or(a, b) => Expr::Or(under(a), under(b)),
            Expr::Not(a) => Expr::Not(under(a)),
            Expr::Term(Term::Path(path)) => Expr::Term(Term::Path(path.under(place))),
            term => term,
        }
    }
}
