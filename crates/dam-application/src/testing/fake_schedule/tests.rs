use super::FakeSchedule;
use crate::testing::contract::{schedule_contract, scheduled_run};

#[test]
fn the_fake_schedule_keeps_the_schedule_contract() {
    schedule_contract(&FakeSchedule::default(), &scheduled_run());
}
