use super::Base;
use crate::Path;

impl Base {
    pub fn owned_folder(&self) -> Option<Path> {
        self.path.join(&self.subject.replace('/', "-")).ok()
    }
}

#[cfg(test)]
mod tests;
