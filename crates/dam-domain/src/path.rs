use std::fmt;

#[derive(Clone, Debug, Default, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct Path(String);

#[derive(Debug, PartialEq, Eq)]
pub enum PathError {
    EmptySegment,
    BadCharacter(char),
    RelativeSegment,
}

impl Path {
    pub fn root() -> Path {
        Path(String::new())
    }

    pub fn parse(text: &str) -> Result<Path, PathError> {
        let trimmed = text.trim_matches('/');
        if trimmed.is_empty() {
            return Ok(Path::root());
        }
        let mut out = String::with_capacity(trimmed.len() + 1);
        for segment in trimmed.split('/') {
            if segment.is_empty() {
                return Err(PathError::EmptySegment);
            }
            if segment == "." || segment == ".." {
                return Err(PathError::RelativeSegment);
            }
            out.push_str(segment);
            out.push('/');
        }
        Ok(Path(out))
    }

    pub fn parent(&self) -> Option<Path> {
        if self.0.is_empty() {
            return None;
        }
        let without_trailing = &self.0[..self.0.len() - 1];
        match without_trailing.rfind('/') {
            Some(i) => Some(Path(self.0[..=i].to_string())),
            None => Some(Path::root()),
        }
    }

    pub fn is_within(&self, ancestor: &Path) -> bool {
        self.0.starts_with(&ancestor.0)
    }

    pub fn join(&self, segment: &str) -> Result<Path, PathError> {
        if segment.is_empty() {
            return Err(PathError::EmptySegment);
        }
        if segment == "." || segment == ".." {
            return Err(PathError::RelativeSegment);
        }
        if let Some(bad) = segment.chars().find(|c| *c == '/') {
            return Err(PathError::BadCharacter(bad));
        }
        Ok(Path(format!("{}{}/", self.0, segment)))
    }

    pub fn rebase(&self, from: &Path, to: &Path) -> Option<Path> {
        let rest = self.0.strip_prefix(&from.0)?;
        Some(Path(format!("{}{rest}", to.0)))
    }

    pub fn depth(&self) -> usize {
        self.0.matches('/').count()
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for Path {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

impl fmt::Display for PathError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            PathError::EmptySegment => f.write_str("a path segment cannot be empty"),
            PathError::BadCharacter(c) => write!(f, "a path segment cannot hold {c:?}"),
            PathError::RelativeSegment => f.write_str("a path segment cannot be \".\" or \"..\""),
        }
    }
}

impl std::error::Error for PathError {}

#[cfg(test)]
mod tests;
