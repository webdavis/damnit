use dam_domain::{Change, Field, Object, changed_fields};

use super::values::cells;
use super::{ChangeRow, what_changed};
use crate::view::Line;

impl ChangeRow<'_> {
    pub(in crate::commands::status) fn own_lines(&self, indent: &str, owner: &str) -> Vec<Line> {
        self.lines(indent, self.oid.short(), owner, &[])
    }

    pub(in crate::commands::status) fn lines(
        &self,
        indent: &str,
        id: &str,
        owner: &str,
        tail: &[String],
    ) -> Vec<Line> {
        let sender = self.sender(owner).to_string();
        let Change::Update { before, after } = self.change else {
            let mut row = vec![format!("{indent}{}", what_changed(self.change, id))];
            row.extend(path_cell(self.change.object()));
            row.push(sender);
            row.extend_from_slice(tail);
            return vec![Line::Row(row.join("  "))];
        };
        let fields = changed_fields(before, after);
        let lead = format!("{indent}changed  {id}  ");
        let mut minus = vec![format!("{lead}-{}", before.base().subject)];
        minus.extend(cells(before, &fields));
        minus.extend(path_cell(before));
        minus.push(sender);
        minus.extend_from_slice(tail);
        let pad = " ".repeat(lead.chars().count());
        let mut plus = vec![format!("{pad}+{}", after.base().subject)];
        plus.extend(cells(after, &fields));
        if fields.contains(&Field::Path) {
            plus.extend(path_cell(after));
        }
        vec![Line::Row(minus.join("  ")), Line::Row(plus.join("  "))]
    }
}

fn path_cell(object: &Object) -> Option<String> {
    let path = object.base().path.as_str();
    (!path.is_empty()).then(|| path.to_string())
}
