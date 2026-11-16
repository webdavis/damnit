use clap::Parser;
use clap::error::ErrorKind;

use super::Cli;
use crate::output::Format;

fn refused(line: &[&str]) -> clap::Error {
    let error = Cli::try_parse_from(line).unwrap_err();
    assert_eq!(error.exit_code(), 2, "{line:?}");
    error
}

#[test]
fn output_chooses_json_toon_or_yaml_on_any_command() {
    for (line, format) in [
        (&["dam", "ls", "-o", "json"][..], Format::Json),
        (&["dam", "-o", "toon", "ls"][..], Format::Toon),
        (&["dam", "status", "--output", "yaml"][..], Format::Yaml),
        (&["dam", "ls", "--output=json"][..], Format::Json),
        (&["dam", "ls", "-oyaml"][..], Format::Yaml),
    ] {
        assert_eq!(Cli::try_parse_from(line).unwrap().output, Some(format));
    }
    assert_eq!(Cli::try_parse_from(["dam", "ls"]).unwrap().output, None);
}

#[test]
fn an_unknown_output_names_the_three_values() {
    for value in ["xml", "human", "JSON", ""] {
        let error = refused(&["dam", "ls", "-o", value]);
        assert!(
            error
                .to_string()
                .contains("expected one of: json, toon, yaml"),
            "{error}"
        );
    }
}

#[test]
fn output_given_twice_or_without_a_value_is_refused() {
    let twice = refused(&["dam", "ls", "-o", "json", "-o", "yaml"]);
    assert_eq!(twice.kind(), ErrorKind::ArgumentConflict, "{twice}");
    let bare = refused(&["dam", "ls", "-o"]);
    assert_eq!(bare.kind(), ErrorKind::InvalidValue, "{bare}");
}
