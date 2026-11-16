use std::fmt;

use dam_domain::{Oid, Path};

pub(super) fn parent_not_known(
    f: &mut fmt::Formatter<'_>,
    oid: &Oid,
    unsettled: &[Oid],
    namesakes: &[Oid],
    folder: &Path,
    held_by_a_remote: bool,
) -> fmt::Result {
    let first = unsettled.first().map_or("", |o| o.short());
    let held = match unsettled.len() {
        0 | 1 => first.to_string(),
        n => format!("{first} and {} more objects in {}", n - 1, folder.as_str()),
    };
    let pull = if held_by_a_remote {
        "run: dam pull, or "
    } else {
        ""
    };
    write!(
        f,
        "dam does not know whether {} holds {held}, so it cannot tell what moves with {}; {pull}name it: dam mv {first} --parent {}",
        either(oid, namesakes),
        oid.short(),
        oid.short()
    )
}

fn either(oid: &Oid, namesakes: &[Oid]) -> String {
    let shorts: Vec<&str> = std::iter::once(oid)
        .chain(namesakes)
        .map(Oid::short)
        .collect();
    match shorts.split_last() {
        Some((last, rest)) if !rest.is_empty() => format!("{} or {last}", rest.join(", ")),
        _ => shorts.concat(),
    }
}
