use crate::support;

use std::process::Output;

use support::small_config::small_config;

const ESCAPE: u8 = 0x1b;

fn without_escapes(text: &[u8]) -> Vec<u8> {
    let mut plain = Vec::new();
    let mut bytes = text.iter();
    while let Some(byte) = bytes.next() {
        match *byte {
            ESCAPE => {
                bytes.by_ref().find(|b| **b == b'm');
            }
            other => plain.push(other),
        }
    }
    plain
}

fn forced(sb: &support::sandbox::Sandbox, args: &[&str], extra: &[(&str, &str)]) -> Output {
    let mut command = sb.command(args);
    command.env("CLICOLOR_FORCE", "1");
    for (name, value) in extra {
        command.env(name, value);
    }
    command.output().unwrap()
}

#[test]
fn piped_output_holds_no_color() {
    let _guard = support::guard("piped_output_holds_no_color");
    let out = small_config().output(&["config", "list"]);
    assert!(out.status.success(), "{out:?}");
    assert!(!out.stdout.contains(&ESCAPE));
}

#[test]
fn forced_color_paints_the_same_layout() {
    let _guard = support::guard("forced_color_paints_the_same_layout");
    let sb = small_config();
    for line in [
        &["config", "list"][..],
        &["config", "get", "remote.todoist"],
    ] {
        let plain = sb.output(line);
        let painted = forced(&sb, line, &[]);
        assert!(painted.status.success(), "{painted:?}");
        assert!(painted.stdout.contains(&ESCAPE), "{line:?}");
        assert_eq!(without_escapes(&painted.stdout), plain.stdout, "{line:?}");
    }
}

#[test]
fn no_color_and_the_flag_each_win_over_forced_color() {
    let _guard = support::guard("no_color_and_the_flag_each_win_over_forced_color");
    let sb = small_config();
    let plain = sb.output(&["config", "list"]);
    for out in [
        forced(&sb, &["config", "list"], &[("NO_COLOR", "1")]),
        forced(&sb, &["config", "list", "--no-color"], &[]),
    ] {
        assert_eq!(out.stdout, plain.stdout, "{out:?}");
    }
}

#[test]
fn machine_answers_hold_no_color_when_it_is_forced() {
    let _guard = support::guard("machine_answers_hold_no_color_when_it_is_forced");
    let sb = small_config();
    for line in [
        &["config", "list", "--json"][..],
        &["config", "list", "--toon"],
        &["config", "get", "owner", "--json"],
        &["config", "get", "owner", "--toon"],
    ] {
        let out = forced(&sb, line, &[]);
        assert!(out.status.success(), "{out:?}");
        assert!(!out.stdout.contains(&ESCAPE), "{line:?}");
    }
}
