use clap::Parser;
use dam_application::Refusal;

use super::*;
use crate::args::Cli;

#[test]
fn a_token_may_not_start_stop_enable_or_disable_background_sync() {
    for verb in ["start", "stop", "enable", "disable"] {
        let command = Cli::try_parse_from(["dam", "sync", verb, "todoist"])
            .unwrap()
            .command;
        let refused = refuse_any_token_to_an_owner_only_verb(true, &command);
        let named: &'static str = Box::leak(format!("sync {verb}").into_boxed_str());
        assert_eq!(refused, Err(Refusal::NotOpenToTokens(named)));
        assert_eq!(
            refuse_any_token_to_an_owner_only_verb(false, &command),
            Ok(())
        );
    }
}
