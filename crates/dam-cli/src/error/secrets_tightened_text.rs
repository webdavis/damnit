use std::fmt;

use dam_application::SecretsTightened;

pub(crate) struct SecretsTightenedText<'a>(pub(crate) &'a SecretsTightened);

impl fmt::Display for SecretsTightenedText<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "secrets.lua was mode {:o}; dam wrote it 600, since it holds secrets",
            self.0.was
        )
    }
}
