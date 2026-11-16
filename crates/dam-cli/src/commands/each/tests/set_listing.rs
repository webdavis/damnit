use dam_domain::{Grant, Path, Permission};

use super::as_one::a_waiting_set;
use super::*;

fn reader_of(paths: &[&str]) -> Box<dam_application::Holder> {
    let mut reader = crate::testing::holding(&[]);
    reader.name = "reader".into();
    reader.grants.grants = vec![Grant {
        allow: [Permission::TaskRead, Permission::PendingRead].into(),
        paths: Some(paths.iter().map(|p| Path::parse(p).unwrap()).collect()),
        ..Grant::default()
    }];
    reader
}

#[test]
fn status_shows_a_waiting_set_as_one_row_with_its_count() {
    let (ctx, set, member) = a_waiting_set();
    let out = dam(&ctx, &["status"]).unwrap();
    let rows: Vec<&str> = out
        .human
        .lines()
        .filter(|l| l.contains(&set[..7]))
        .collect();
    assert_eq!(rows.len(), 1, "{}", out.human);
    assert!(rows[0].contains("3 changes") && rows[0].contains("tool"));
    assert!(rows[0].contains(&format!("dam show {}", &set[..7])));
    assert!(!out.human.contains(&member[..7]));
    assert_eq!(out.data["sets"][0]["set"], set.as_str());
    assert_eq!(out.data["sets"][0]["items"].as_array().unwrap().len(), 3);
    assert_eq!(out.data["waiting"].as_array().unwrap().len(), 0);
}

#[test]
fn show_lists_each_change_in_a_set_and_how_to_take_or_decline_it() {
    let (ctx, set, _) = a_waiting_set();
    let out = dam(&ctx, &["show", &set[..7]]).unwrap();
    assert_eq!(out.data["set"], set.as_str());
    assert_eq!(out.data["items"].as_array().unwrap().len(), 3);
    assert!(out.human.contains(&format!("dam add {}", &set[..7])));
    assert!(out.human.contains(&format!("dam rm {}", &set[..7])));
}

#[test]
fn a_token_that_sees_part_of_a_set_learns_nothing_of_it() {
    let (mut ctx, set, member) = a_waiting_set();
    ctx.access = reader_of(&["work"]);
    let status = dam(&ctx, &["status"]).unwrap();
    assert!(!status.human.contains(&set[..7]) && !status.human.contains(&member[..7]));
    assert_eq!(status.data["sets"].as_array().unwrap().len(), 0);
    for named in [&set[..], &set[..7]] {
        for verb in ["show", "add", "rm"] {
            let refused = dam(&ctx, &[verb, named]).unwrap_err().to_string();
            assert!(
                !refused.contains("set") && !refused.contains("change"),
                "{verb} {named}: {refused}"
            );
        }
    }
}

#[test]
fn a_token_that_sees_all_of_a_set_finds_it() {
    let (mut ctx, set, _) = a_waiting_set();
    ctx.access = reader_of(&["work", "home"]);
    let out = dam(&ctx, &["show", &set]).unwrap();
    assert_eq!(out.data["items"].as_array().unwrap().len(), 3);
}
