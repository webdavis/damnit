use dam_protocol::WireObject;

use crate::map::Several;

pub(super) fn several(
    Several(count): Several,
    kind: &str,
    name: &str,
    under: &str,
    object: &WireObject,
) -> String {
    let under = match under {
        "" => String::new(),
        path => format!(" under {path}"),
    };
    let neither = if count == 2 {
        "neither"
    } else {
        "none of them"
    };
    let short = object.oid.get(..7).unwrap_or(&object.oid);
    format!(
        "{count} Todoist {kind} are named {name:?}{under} and dam named {neither}; \
         run: dam pull, then dam mv {short} --parent OID"
    )
}
