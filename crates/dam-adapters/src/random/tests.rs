use super::*;
use dam_application::Randomness;

#[test]
fn two_fills_differ() {
    let mut a = [0u8; 16];
    let mut b = [0u8; 16];
    OsRandom.fill(&mut a);
    OsRandom.fill(&mut b);
    assert_ne!(a, b);
}
