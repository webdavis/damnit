use dam_application::Randomness;
use dam_application::testing::FixedRandom;

use crate::args::CreateArgs;
use crate::testing::FakeConfigFile;

fn tool_reading_tasks() -> CreateArgs {
    CreateArgs {
        name: "tool".into(),
        allow: vec!["task:read".into()],
        paths: vec![],
        labels: vec![],
        own: false,
        remotes: vec![],
        expires: None,
        via: vec![],
        require: vec![],
    }
}

fn fills_made_by(random: &FixedRandom) -> u8 {
    let mut next = [0u8; 1];
    random.fill(&mut next);
    next[0] - 1
}

#[test]
fn a_file_that_cannot_take_the_token_is_refused_before_one_is_made() {
    let file = FakeConfigFile::refusing_tokens("secrets.lua returns a table built in code");
    let random = FixedRandom::new(1);
    let refused = super::run(&file, &random, tool_reading_tasks());
    let message = refused.err().map(|e| e.to_string()).unwrap_or_default();
    assert!(message.contains("built in code"), "{message}");
    assert_eq!(fills_made_by(&random), 0);
    assert!(file.tokens_added().is_empty());
}

#[test]
fn a_file_that_can_take_the_token_gets_it() {
    let file = FakeConfigFile::default();
    let random = FixedRandom::new(1);
    let created = super::run(&file, &random, tool_reading_tasks());
    assert!(created.is_ok());
    assert_eq!(file.tokens_added(), ["tool"]);
}
