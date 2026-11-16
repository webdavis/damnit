use dam_domain::Path;

use super::SelectArgs;

impl SelectArgs {
    pub(crate) fn under(mut self, place: &Path) -> SelectArgs {
        self.typed = self.typed.under(place);
        self
    }
}
