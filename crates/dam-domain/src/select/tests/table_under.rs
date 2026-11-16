use super::tables::{either, many, one, table};
use crate::Path;

#[test]
fn every_path_moves_under_the_place_the_rest_stays() {
    let place = Path::parse("work/app/").unwrap();
    let t = either(
        table(&[
            ("path", many(&["docs/", "not:old/,tmp/"])),
            ("labels", one("x/")),
        ]),
        vec![table(&[("path", one("=a,b/"))])],
    );
    let moved = either(
        table(&[
            (
                "path",
                many(&["work/app/docs/", "not:work/app/old/,work/app/tmp/"]),
            ),
            ("labels", one("x/")),
        ]),
        vec![table(&[("path", one("=work/app/a,b/"))])],
    );
    assert_eq!(t.under(&place), moved);
    assert_eq!(t.under(&Path::root()), t);
}
