use crate::nvim_fixtures::assert_fixture;
use crate::support;

use support::sandbox::Sandbox;
use support::schema::assert_matches_schema;

#[test]
fn the_doctor_fixture_is_what_dam_doctor_answers_in_a_fresh_sandbox() {
    let _guard = support::guard("the_doctor_fixture_is_what_dam_doctor_answers_in_a_fresh_sandbox");
    let sb = Sandbox::new();
    let answer = sb.doctor(&["doctor", "-o", "json"]);
    assert_matches_schema(
        &serde_json::from_str(&answer.out).unwrap(),
        "doctor.schema.json",
    );
    let sandbox = sb.dir.path().to_str().unwrap();
    assert_fixture("doctor.json", &answer.out.replace(sandbox, "<sandbox>"));
}
