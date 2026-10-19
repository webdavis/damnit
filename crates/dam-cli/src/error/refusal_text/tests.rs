mod samples;

use super::*;
use dam_domain::Oid;
use samples::{RULES, every_refusal_with_its_text, rule_number};

pub(super) fn oid(byte: u8) -> Oid {
    Oid::generate(&mut |x: &mut [u8]| x.fill(byte))
}

#[test]
fn every_rule_has_a_sample_with_its_text() {
    let mut numbers: Vec<usize> = every_refusal_with_its_text()
        .iter()
        .map(|(refusal, _)| rule_number(refusal))
        .collect();
    numbers.dedup();
    assert_eq!(numbers, (0..RULES).collect::<Vec<usize>>());
}

#[test]
fn every_refusal_reads_as_its_one_line() {
    for (refusal, text) in every_refusal_with_its_text() {
        assert_eq!(RefusalText(&refusal).to_string(), text);
        assert!(!text.contains('\n'), "{text:?}");
    }
}

#[test]
fn a_blocked_task_names_its_open_children_and_dependencies_in_one_line() {
    let blocked = |dependencies, children| {
        RefusalText(&Refusal::Blocked {
            oid: oid(1),
            blockers: Blockers {
                dependencies,
                children,
            },
        })
        .to_string()
    };
    assert_eq!(
        blocked(vec![], vec![oid(2), oid(3), oid(4)]),
        "0101010 has 3 open children: 0202020, 0303030, 0404040; pass --force to complete it anyway"
    );
    assert_eq!(
        blocked(vec![oid(2)], vec![]),
        "0101010 has 1 open dependency: 0202020; pass --force to complete it anyway"
    );
    assert_eq!(
        blocked(vec![oid(3), oid(4)], vec![oid(2)]),
        "0101010 has 1 open child: 0202020 and 2 open dependencies: 0303030, 0404040; pass --force to complete it anyway"
    );
}
