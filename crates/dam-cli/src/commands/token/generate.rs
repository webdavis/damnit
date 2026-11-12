use dam_application::Randomness;
use dam_domain::Token;
use serde::Serialize;

use crate::error::CliError;
use crate::output::Report;
use crate::view::{Human, View, report};

#[derive(Serialize)]
struct TokenGenerated {
    key_id: String,
    value: String,
}

impl View for TokenGenerated {
    fn human(&self) -> Human {
        Human::Lines(vec![self.value.clone()])
    }
}

pub(super) fn run(random: &dyn Randomness) -> Result<Report, CliError> {
    let token = Token::generate(&mut |b: &mut [u8]| random.fill(b));
    report(&TokenGenerated {
        key_id: token.key_id().to_string(),
        value: token.expose(),
    })
}
