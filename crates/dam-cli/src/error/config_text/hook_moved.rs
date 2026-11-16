use std::fmt;

pub(super) fn hook_moved(
    f: &mut fmt::Formatter<'_>,
    config: &str,
    events: &[String],
) -> fmt::Result {
    let moves = events
        .iter()
        .map(|event| format!("hook.{event} moves to hooks/{event}.lua"))
        .collect::<Vec<_>>();
    match moves.is_empty() {
        true => write!(
            f,
            "{config}: the hook table is gone; each hook moves to hooks/<event>.lua"
        ),
        false => write!(f, "{config}: {}", moves.join(", ")),
    }?;
    write!(
        f,
        "; put each hook's function or table after return in its own file beside {config}, \
         remove the hook table, then run dam config validate"
    )
}
