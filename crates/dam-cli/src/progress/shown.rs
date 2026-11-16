#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Shown {
    Hidden,
    Animated,
    Records,
}

#[derive(Clone, Copy, Debug)]
pub(crate) struct Asked {
    pub(crate) setting: bool,
    pub(crate) progress: bool,
    pub(crate) no_progress: bool,
    pub(crate) quiet: bool,
}

pub(crate) fn shown(asked: Asked, terminal: bool) -> Shown {
    match (asked, terminal) {
        (
            Asked { quiet: true, .. }
            | Asked {
                no_progress: true, ..
            },
            _,
        ) => Shown::Hidden,
        (Asked { progress: true, .. }, false) => Shown::Records,
        (Asked { progress: true, .. }, true) | (Asked { setting: true, .. }, true) => {
            Shown::Animated
        }
        _ => Shown::Hidden,
    }
}

#[cfg(test)]
mod tests;
