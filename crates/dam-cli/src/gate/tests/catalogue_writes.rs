use dam_application::{Owner, Refusal};
use dam_domain::Permission;

use super::{allowing, command, holding};
use crate::gate::admit;

#[test]
fn only_the_owner_creates_or_removes_a_filter_or_a_category() {
    let everything = allowing(&Permission::ALL);
    for (line, verb) in [
        ("filter create f --due today", "filter create"),
        ("filter remove f", "filter remove"),
        ("category create c --values a", "category create"),
        ("category remove c", "category remove"),
        ("filter create f --due today --global", "filter create"),
        ("category remove c --global", "category remove"),
    ] {
        assert_eq!(admit(&Owner, &command(line)), Ok(()), "{line}");
        assert_eq!(
            admit(&everything, &command(line)),
            Err(Refusal::NotOpenToTokens(verb))
        );
        assert_eq!(
            admit(&holding(vec![]), &command(line)),
            Err(Refusal::NotOpenToTokens(verb))
        );
    }
}
