use super::Base;
use crate::Path;

impl Base {
    pub fn owned_folder(&self) -> Option<Path> {
        self.path.join(&self.subject.replace('/', "-")).ok()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Oid;

    fn base(subject: &str, path: &str) -> Base {
        let mut base = Base::new(Oid::generate(&mut |b: &mut [u8]| b.fill(1)), subject);
        base.path = Path::parse(path).unwrap();
        base
    }

    #[test]
    fn an_object_owns_the_folder_named_by_its_subject_under_its_path() {
        assert_eq!(
            base("milk", "work/now").owned_folder().unwrap().as_str(),
            "work/now/milk/"
        );
        assert_eq!(base("milk", "").owned_folder().unwrap().as_str(), "milk/");
    }

    #[test]
    fn a_slash_in_the_subject_becomes_a_dash() {
        assert_eq!(base("a/b", "").owned_folder().unwrap().as_str(), "a-b/");
    }

    #[test]
    fn a_subject_that_cannot_name_a_folder_owns_none() {
        assert_eq!(base("", "work").owned_folder(), None);
        assert_eq!(base("..", "work").owned_folder(), None);
    }
}
