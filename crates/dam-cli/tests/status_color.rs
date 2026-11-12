mod support;

use support::hidden::text;
use support::sandbox::Sandbox;

const RED: &str = "\x1b[31m";
const GREEN: &str = "\x1b[32m";

fn retitled() -> Sandbox {
    let sb = Sandbox::new();
    let short = sb.new_object(&["Pay rent"]);
    assert!(sb.dam(&["add", "--all"]).0);
    assert!(sb.dam(&["commit", "-m", "plan"]).0);
    assert!(sb.dam(&["edit", &short, "--subject", "Pay the rent"]).0);
    sb
}

fn forced(sb: &Sandbox, args: &[&str], no_color: &str) -> String {
    let out = sb
        .command(args)
        .env("CLICOLOR_FORCE", "1")
        .env("NO_COLOR", no_color)
        .output()
        .unwrap();
    assert!(out.status.success(), "{out:?}");
    text(&out.stdout)
}

#[test]
fn forced_color_paints_the_minus_line_red_and_the_plus_line_green() {
    let _guard = support::guard("forced_color_paints_the_minus_line_red_and_the_plus_line_green");
    let sb = retitled();
    let status = forced(&sb, &["status"], "");
    let line = |sign: &str| {
        status
            .lines()
            .find(|l| l.contains(sign))
            .unwrap()
            .to_string()
    };
    assert!(line("-Pay rent").starts_with(RED), "{status:?}");
    assert!(line("+Pay the rent").starts_with(GREEN), "{status:?}");
    assert!(status.starts_with("Not staged:\n"), "{status:?}");
}

#[test]
fn a_pipe_no_color_and_the_no_color_flag_each_turn_color_off() {
    let _guard = support::guard("a_pipe_no_color_and_the_no_color_flag_each_turn_color_off");
    let sb = retitled();
    let piped = text(&sb.output(&["status"]).stdout);
    let no_color_set = forced(&sb, &["status"], "1");
    let flagged = forced(&sb, &["status", "--no-color"], "");
    for status in [&piped, &no_color_set, &flagged] {
        assert!(!status.contains('\x1b'), "{status:?}");
        assert!(status.contains("+Pay the rent"), "{status}");
    }
}

#[test]
fn json_and_toon_are_never_painted() {
    let _guard = support::guard("json_and_toon_are_never_painted");
    let sb = retitled();
    for format in ["--json", "--toon"] {
        let out = forced(&sb, &["status", format], "");
        assert!(!out.contains('\x1b'), "{out:?}");
    }
}
