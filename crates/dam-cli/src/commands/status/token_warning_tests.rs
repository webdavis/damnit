use crate::args::StatusArgs;
use crate::testing::context;
use dam_application::{Standing, TokenConfig};
use dam_domain::Token;

#[test]
fn status_warns_about_each_skipped_token_setting_and_each_disabled_token() {
    let mut ctx = context();
    ctx.config.tokens.warnings =
        vec!["token vpt: unknown permission \"task:teleport\" is skipped".into()];
    ctx.config.tokens.entries = vec![TokenConfig {
        name: "claude".into(),
        value: Token::generate(&mut |b: &mut [u8]| b.fill(1)),
        standing: Standing::Disabled("unknown key weekdays in [token.claude.grant]".into()),
    }];
    let report = super::run_status(&ctx, StatusArgs { full: false }).unwrap();
    let warnings = report.data["warnings"].as_array().unwrap();
    assert_eq!(warnings.len(), 2, "{warnings:?}");
    assert!(
        report
            .human
            .contains("warning: token vpt: unknown permission")
    );
    assert!(report.human.contains(
        "warning: token claude is disabled: unknown key weekdays in [token.claude.grant]"
    ));
}
