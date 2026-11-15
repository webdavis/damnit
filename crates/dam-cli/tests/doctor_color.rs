mod support;

use support::sandbox::Sandbox;

const ESCAPE: char = '\x1b';

fn without_escapes(text: &str) -> String {
    let mut plain = String::new();
    let mut chars = text.chars();
    while let Some(c) = chars.next() {
        match c {
            ESCAPE => {
                chars.by_ref().find(|c| *c == 'm');
            }
            _ => plain.push(c),
        }
    }
    plain
}

#[test]
fn forced_color_paints_the_same_report_and_a_pipe_gets_none() {
    let _guard = support::guard("forced_color_paints_the_same_report_and_a_pipe_gets_none");
    let sb = Sandbox::new();
    sb.write_config_lua("return {\n");
    let plain = sb.doctor(&["doctor"]);
    assert!(!plain.out.contains(ESCAPE), "{}", plain.out);
    let painted = sb
        .command(&["doctor"])
        .env("CLICOLOR_FORCE", "1")
        .output()
        .unwrap();
    assert_eq!(painted.status.code(), Some(1));
    let painted = String::from_utf8(painted.stdout).unwrap();
    assert!(painted.contains(ESCAPE), "{painted}");
    assert_eq!(without_escapes(&painted), plain.out);
    let out = sb
        .command(&["doctor", "--no-color"])
        .env("CLICOLOR_FORCE", "1")
        .output()
        .unwrap();
    assert_eq!(String::from_utf8(out.stdout).unwrap(), plain.out);
    let out = sb
        .command(&["doctor"])
        .env("CLICOLOR_FORCE", "1")
        .env("NO_COLOR", "1")
        .output()
        .unwrap();
    assert_eq!(String::from_utf8(out.stdout).unwrap(), plain.out);
}
