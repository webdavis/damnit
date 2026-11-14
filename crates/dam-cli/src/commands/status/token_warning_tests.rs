use crate::args::StatusArgs;
use crate::testing::context;
use dam_application::{Misread, Place, Standing, TokenConfig, TokenWarning};
use dam_domain::Token;

#[test]
fn status_warns_about_each_skipped_token_setting_and_each_disabled_token() {
    let mut ctx = context();
    ctx.config.tokens.warnings = vec![TokenWarning::UnknownPermission {
        name: "tool".into(),
        word: "task:teleport".into(),
    }];
    ctx.config.tokens.entries = vec![TokenConfig {
        name: "claude".into(),
        value: Token::generate(&mut |b: &mut [u8]| b.fill(1)),
        standing: Standing::Disabled(
            Misread::UnknownKey {
                key: "weekdays".into(),
                at: Place {
                    file: "secrets.lua".into(),
                    table: Some("token.claude.grant".into()),
                },
            }
            .into(),
        ),
    }];
    let report = super::run_status(&ctx, StatusArgs { full: false }).unwrap();
    let warnings = report.data["warnings"].as_array().unwrap();
    assert_eq!(warnings.len(), 2, "{warnings:?}");
    assert!(
        report
            .human
            .contains("warning: token tool: unknown permission")
    );
    assert!(report.human.contains(
        "warning: token claude is disabled: unknown key weekdays in token.claude.grant of secrets.lua"
    ));
}
