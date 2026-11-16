use dam_adapters::checked_out_branch;
use dam_application::ProjectSettings;
use dam_domain::{Categories, Expr, Term, parse_query};

use crate::project_scope::Standing;

const NOT_A_LABEL: &str = "dam: warning: the checked-out git branch's name is not a label dam can use, so the task has no branch label";

pub(super) fn branch_of(
    standing: &Standing,
    rules: &ProjectSettings,
) -> (Option<String>, Option<String>) {
    let branch = match standing {
        Standing::Inside(project) if rules.branch_label => checked_out_branch(&project.folder),
        _ => None,
    };
    match branch {
        Some(name) if usable(&name) => (Some(name), None),
        Some(_) => (None, Some(NOT_A_LABEL.into())),
        None => (None, None),
    }
}

fn usable(name: &str) -> bool {
    let plain = !name.chars().any(|c| c.is_control() || c.is_whitespace());
    let read = parse_query(&format!("@{name}"), &Categories::default());
    plain && matches!(read, Ok(Expr::Term(Term::Label(label))) if label == name)
}
