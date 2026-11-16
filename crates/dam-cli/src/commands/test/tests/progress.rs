use dam_application::testing::Said;

use super::{FAKE, FAKE_TOKEN, Files, Helpers};

fn signing_in_to_fake() -> Said {
    Said::Started("sign_in@fake: signing in to fake ".into(), None)
}

#[test]
fn signing_in_to_a_remote_is_a_step_that_finishes() {
    let files = Files::new(FAKE, FAKE_TOKEN);
    files.report(&Helpers::default(), &[]);
    assert_eq!(
        *files.progress.said.borrow(),
        [signing_in_to_fake(), Said::Finished]
    );
}

#[test]
fn a_refused_sign_in_ends_the_step_as_failed() {
    let refused = "return { schema = 1, remote = { fake = { api_token = \"wrong\" } } }\n";
    let files = Files::new(FAKE, refused);
    files.report(&Helpers::default(), &[]);
    assert_eq!(
        *files.progress.said.borrow(),
        [signing_in_to_fake(), Said::Failed]
    );
}
