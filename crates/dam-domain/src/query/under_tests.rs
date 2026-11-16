use crate::{Categories, Path};

use super::{Expr, Term, parse};

fn query(text: &str) -> Expr {
    parse(text, &Categories::default()).unwrap()
}

#[test]
fn every_path_term_starts_from_the_place_given() {
    let project = Path::parse("damnit/").unwrap();
    assert_eq!(
        query("path:cli/ | !(path:docs/ & @x)").under(&project),
        query("path:damnit/cli/ | !(path:damnit/docs/ & @x)")
    );
}

#[test]
fn terms_other_than_path_are_kept() {
    let project = Path::parse("damnit/").unwrap();
    assert_eq!(
        query("@x & due:today").under(&project),
        query("@x & due:today")
    );
    assert_eq!(
        Expr::Term(Term::Done).under(&project),
        Expr::Term(Term::Done)
    );
}
