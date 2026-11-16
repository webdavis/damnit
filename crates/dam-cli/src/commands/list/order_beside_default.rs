use crate::args::selection::ordering::OrderArgs;
use crate::error::CliError;

pub(super) fn refused(order: &OrderArgs) -> Option<CliError> {
    let mut words: Vec<String> = order
        .sort
        .iter()
        .map(|text| format!("--sort {text}"))
        .collect();
    words.extend(order.limit.map(|n| format!("--limit {n}")));
    words.extend(order.offset.map(|n| format!("--offset {n}")));
    (!words.is_empty()).then(|| {
        CliError::Usage(format!(
            "this project's list default is query text, which --sort, --limit and --offset cannot follow; run dam list --done false {}",
            words.join(" ")
        ))
    })
}
