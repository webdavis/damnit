use dam_application::{ConfigPlace, SetChanges};
use serde::Serialize;
use serde_json::{Value, json};

use crate::commands::each::SetView;
use crate::view::{Human, Line, View};

pub(super) struct Place {
    pub(super) place: ConfigPlace,
    pub(super) written: bool,
}

impl From<ConfigPlace> for Place {
    fn from(place: ConfigPlace) -> Place {
        Place {
            place,
            written: false,
        }
    }
}

#[derive(Serialize)]
pub(super) struct Migrated<'a> {
    #[serde(flatten)]
    set: SetView<'a>,
    config: Vec<Value>,
    #[serde(skip)]
    places: Vec<Place>,
    #[serde(skip)]
    updating: bool,
}

impl<'a> Migrated<'a> {
    pub(super) fn of(
        changes: &'a SetChanges,
        owner: &'a str,
        places: Vec<Place>,
        updating: bool,
    ) -> Migrated<'a> {
        Migrated {
            set: SetView::of(changes, owner),
            config: places.iter().map(document).collect(),
            places,
            updating,
        }
    }
}

fn document(place: &Place) -> Value {
    match &place.place {
        ConfigPlace::Category { name, values } => json!({
            "key": format!("category.{name}.values"), "value": values, "written": place.written
        }),
        ConfigPlace::Filter { name, query } => json!({
            "key": format!("filter.{name}.query"), "value": query, "written": place.written
        }),
        ConfigPlace::HandWritten { name } => json!({
            "key": format!("filter.{name}"), "by_hand": true, "written": false
        }),
    }
}

pub(super) fn fix(place: &ConfigPlace) -> Option<String> {
    match place {
        ConfigPlace::Category { name, values } => Some(format!(
            "dam config set category.{name}.values {}",
            quoted(&values.join(", "))
        )),
        ConfigPlace::Filter { name, query } => Some(format!(
            "dam config set filter.{name}.query {}",
            quoted(query)
        )),
        ConfigPlace::HandWritten { .. } => None,
    }
}

fn quoted(text: &str) -> String {
    format!("'{}'", text.replace('\'', r"'\''"))
}

impl Migrated<'_> {
    fn place_line(&self, place: &ConfigPlace) -> String {
        match place {
            ConfigPlace::HandWritten { name } => format!(
                "  filter {name} runs a match function → edit filter.{name} in config.lua by hand"
            ),
            ConfigPlace::Category { name, values } if self.updating => {
                format!("  category.{name}.values = {}", values.join(", "))
            }
            ConfigPlace::Filter { name, query } if self.updating => {
                format!("  filter.{name}.query = {query}")
            }
            ConfigPlace::Category { name, .. } => {
                format!("  category {name} → {}", fix(place).unwrap_or_default())
            }
            ConfigPlace::Filter { name, .. } => {
                format!("  filter {name} → {}", fix(place).unwrap_or_default())
            }
        }
    }

    fn heading(&self) -> String {
        let rewritten = self
            .places
            .iter()
            .filter(|p| !matches!(p.place, ConfigPlace::HandWritten { .. }))
            .count();
        let places = |n: usize| {
            if n == 1 {
                "1 place".to_string()
            } else {
                format!("{n} places")
            }
        };
        match (self.updating, self.set.is_preview()) {
            (false, _) => format!(
                "config.lua still names an old label in {}:",
                places(self.places.len())
            ),
            (true, true) => format!("config.lua would change in {}:", places(rewritten)),
            (true, false) => format!("config.lua updated in {}:", places(rewritten)),
        }
    }
}

impl View for Migrated<'_> {
    fn human(&self) -> Human {
        let mut lines = match self.set.human() {
            Human::Mixed(lines) => lines,
            Human::Lines(lines) => lines.into_iter().map(Line::Whole).collect(),
            Human::Table(_) | Human::Styled { .. } => Vec::new(),
        };
        if !self.places.is_empty() {
            lines.push(Line::Whole(self.heading()));
            lines.extend(
                self.places
                    .iter()
                    .map(|p| Line::Whole(self.place_line(&p.place))),
            );
        }
        Human::Mixed(lines)
    }
}
