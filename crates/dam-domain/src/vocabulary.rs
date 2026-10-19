macro_rules! vocabulary {
    ($name:ident { $($variant:ident => $word:literal),+ $(,)? }) => {
        impl $name {
            pub const ALL: [$name; [$($word),+].len()] = [$($name::$variant),+];

            pub fn as_str(&self) -> &'static str {
                match self {
                    $($name::$variant => $word),+
                }
            }

            pub fn parse(word: &str) -> Option<$name> {
                $name::ALL.into_iter().find(|v| v.as_str() == word)
            }
        }
    };
}

pub(crate) use vocabulary;
