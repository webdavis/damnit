mod support;

use support::answers::answers;
use support::commands::every_command;
use support::sandbox::Sandbox;
use support::schema::{assert_matches_schema, schemas};

fn names(command: &[String]) -> (String, String) {
    (command.join(" "), command.join("-"))
}

#[test]
fn every_command_answers_with_a_document_its_published_schema_accepts() {
    let _guard =
        support::guard("every_command_answers_with_a_document_its_published_schema_accepts")
            .allow_slow("it runs every command of the CLI against its own sandbox");
    let commands = every_command(&Sandbox::new());
    assert!(
        commands.iter().any(|c| names(c).0 == "token create"),
        "{commands:?}"
    );
    for command in commands {
        let (spoken, file) = names(&command);
        let schema = format!("{file}.schema.json");
        assert!(
            schemas().join(&schema).exists(),
            "dam {spoken} has no {schema}"
        );
        let documents = answers(&spoken)
            .unwrap_or_else(|| panic!("dam {spoken} has no answer to check against {schema}"));
        for document in documents {
            assert_matches_schema(&document, &schema);
        }
    }
}
