use std::fmt;

/// Where an object sits in the tree, the way a file sits in a directory.
/// Empty is the root; otherwise segments joined by `/` with a trailing `/`.
#[derive(Clone, Debug, Default, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct Path(String);

#[derive(Debug, PartialEq, Eq)]
pub enum PathError {
    EmptySegment,
    BadCharacter(char),
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
        if let Some(bad) = segment.chars().find(|c| *c == '/') {
            return Err(PathError::BadCharacter(bad));
        }
        Ok(Path(format!("{}{}/", self.0, segment)))
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
        }
    }
}

impl std::error::Error for PathError {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn root_is_empty_and_has_no_parent() {
        assert_eq!(Path::root().as_str(), "");
        assert_eq!(Path::root().parent(), None);
        assert_eq!(Path::root().depth(), 0);
    }

    #[test]
    fn default_is_the_root() {
        assert_eq!(Path::default(), Path::root());
    }

    #[test]
    fn parse_normalizes_a_trailing_slash() {
        assert_eq!(Path::parse("a/b").unwrap().as_str(), "a/b/");
        assert_eq!(Path::parse("a/b/").unwrap().as_str(), "a/b/");
    }

    #[test]
    fn parse_refuses_an_empty_segment() {
        assert_eq!(Path::parse("a//b"), Err(PathError::EmptySegment));
    }

    #[test]
    fn parent_drops_the_last_segment() {
        let p = Path::parse("a/b/c").unwrap();
        assert_eq!(p.parent().unwrap().as_str(), "a/b/");
        assert_eq!(Path::parse("a").unwrap().parent(), Some(Path::root()));
    }

    #[test]
    fn is_within_is_true_for_a_strict_descendant_and_for_self() {
        let a = Path::parse("a").unwrap();
        let ab = Path::parse("a/b").unwrap();
        assert!(ab.is_within(&a));
        assert!(a.is_within(&a));
        assert!(!a.is_within(&ab));
        assert!(ab.is_within(&Path::root()));
    }

    #[test]
    fn join_appends_one_segment() {
        assert_eq!(Path::root().join("x").unwrap().as_str(), "x/");
        assert_eq!(
            Path::parse("a").unwrap().join("b").unwrap().as_str(),
            "a/b/"
        );
        assert_eq!(Path::root().join("a/b"), Err(PathError::BadCharacter('/')));
    }
}
