#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Format {
    Human,
    Quiet,
    Json,
    Toon,
    Yaml,
}

impl Format {
    pub(crate) fn chosen(value: &str) -> Result<Format, String> {
        match value {
            "json" => Ok(Format::Json),
            "toon" => Ok(Format::Toon),
            "yaml" => Ok(Format::Yaml),
            _ => Err("expected one of: json, toon, yaml".into()),
        }
    }

    pub(crate) fn given(matches: &clap::ArgMatches) -> Format {
        matches
            .get_one::<Format>("output")
            .copied()
            .unwrap_or(Format::Human)
    }

    pub(crate) fn quieted(self, quiet: bool) -> Format {
        match self {
            Format::Human if quiet => Format::Quiet,
            _ => self,
        }
    }
}
