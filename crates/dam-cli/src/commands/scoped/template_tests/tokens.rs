use dam_application::testing::{allowed, holder};
use dam_application::{Refusal, TemplateOwner, UseCaseError};
use dam_domain::Permission;

use super::{LookedUp, bug, from, watched};
use crate::commands::scoped::dispatch_in;
use crate::commands::scoped::tests::new_args;
use crate::context::Context;
use crate::error::CliError;
use crate::project_scope::Standing;

fn token(permissions: &[Permission], paths: Option<&str>) -> (Context, LookedUp) {
    let paths = paths.map(|path| [path]);
    let holder = Box::new(holder(vec![allowed(
        permissions,
        paths.as_ref().map(|p| &p[..]),
    )]));
    let (ctx, looked_up) = watched(vec![("bug", TemplateOwner::Person, bug())]);
    let ctx = Context {
        access: holder,
        ..ctx
    };
    (ctx, looked_up)
}

#[test]
fn a_token_that_creates_nothing_is_refused_before_the_template_is_read() {
    let (ctx, looked_up) = token(&[Permission::TaskRead], None);
    let refused = dispatch_in(&ctx, from("bug", new_args("x", "")), Standing::Outside);
    assert!(matches!(
        refused,
        Err(CliError::UseCase(UseCaseError::Refused(
            Refusal::NotPermitted(_)
        )))
    ));
    assert!(looked_up.borrow().is_empty());
}

#[test]
fn a_token_is_checked_against_the_task_the_template_makes() {
    let (elsewhere, _) = token(
        &[Permission::TaskCreate, Permission::TaskRead],
        Some("work"),
    );
    let refused = dispatch_in(
        &elsewhere,
        from("bug", new_args("x", "")),
        Standing::Outside,
    );
    assert_eq!(
        refused.unwrap_err().to_string(),
        "the token does not allow task:create"
    );
    let (within, _) = token(
        &[Permission::TaskCreate, Permission::TaskRead],
        Some("bugs"),
    );
    let made = dispatch_in(&within, from("bug", new_args("x", "")), Standing::Outside).unwrap();
    assert_eq!(made.data["path"], "bugs/");
}

#[test]
fn a_token_asking_for_an_unknown_template_is_told_no_names() {
    let (ctx, _) = token(&[Permission::TaskCreate, Permission::TaskRead], None);
    let Err(CliError::Usage(text)) =
        dispatch_in(&ctx, from("chore", new_args("x", "")), Standing::Outside)
    else {
        panic!("not refused")
    };
    assert_eq!(text, "no template named chore");
}
