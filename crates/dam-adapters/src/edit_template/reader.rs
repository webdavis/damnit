use dam_domain::{Date, When};
use toml::{Table, Value};

pub(super) struct TemplateReader<'a> {
    text: &'a str,
    table: Table,
}

impl<'a> TemplateReader<'a> {
    pub(super) fn parse(text: &'a str) -> Result<TemplateReader<'a>, String> {
        let table: Table = text.parse().map_err(|e: toml::de::Error| {
            match e.span().map(|s| one_based_line_of_byte(text, s.start)) {
                Some(line) => format!("line {line}: {}", e.message()),
                None => e.message().to_string(),
            }
        })?;
        Ok(TemplateReader { text, table })
    }

    pub(super) fn get(&self, key: &str) -> Option<&Value> {
        self.table.get(key)
    }

    pub(super) fn first_unknown_key(&self, known: &[&str]) -> Option<&str> {
        self.table
            .keys()
            .map(String::as_str)
            .find(|key| !known.contains(key))
    }

    pub(super) fn refuse_at_the_keys_line(
        &self,
        key: &str,
        what: impl std::fmt::Display,
    ) -> String {
        match one_based_line_of_bare_key(self.text, key) {
            Some(line) => format!("line {line}: {what}"),
            None => what.to_string(),
        }
    }

    pub(super) fn string(&self, key: &str) -> Result<Option<String>, String> {
        match self.table.get(key) {
            None => Ok(None),
            Some(Value::String(s)) => Ok(Some(s.clone())),
            Some(_) => {
                Err(self.refuse_at_the_keys_line(key, format_args!("{key} must be a string")))
            }
        }
    }

    pub(super) fn when(
        &self,
        key: &str,
        text: &str,
        today: Date,
        tz: &jiff::tz::TimeZone,
    ) -> Result<When, String> {
        When::parse_human(text, today, tz)
            .map_err(|e| self.refuse_at_the_keys_line(key, format_args!("{key}: {e}")))
    }

    pub(super) fn string_array(
        &self,
        key: &str,
        what: &str,
    ) -> Result<Option<Vec<String>>, String> {
        let Some(value) = self.table.get(key) else {
            return Ok(None);
        };
        let items = value
            .as_array()
            .and_then(|a| a.iter().map(|x| x.as_str().map(str::to_string)).collect());
        items
            .map(Some)
            .ok_or_else(|| self.refuse_at_the_keys_line(key, format_args!("{key} must be {what}")))
    }
}

fn one_based_line_of_byte(text: &str, offset: usize) -> usize {
    text[..clamped_to_a_char_boundary(text, offset)]
        .matches('\n')
        .count()
        + 1
}

fn clamped_to_a_char_boundary(text: &str, offset: usize) -> usize {
    let mut end = offset.min(text.len());
    while end > 0 && !text.is_char_boundary(end) {
        end -= 1;
    }
    end
}

fn one_based_line_of_bare_key(text: &str, key: &str) -> Option<usize> {
    text.lines()
        .position(|line| {
            let line = line.trim_start();
            line.strip_prefix(key)
                .is_some_and(|rest| rest.trim_start().starts_with('='))
        })
        .map(|i| i + 1)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn an_offset_inside_a_multibyte_character_still_names_a_line() {
        let text = "subject = \"café\"\nbody = \"\"\n";
        let inside = text.find('é').unwrap() + 1;
        assert_eq!(one_based_line_of_byte(text, inside), 1);
        assert_eq!(one_based_line_of_byte(text, text.len()), 3);
    }

    #[test]
    fn a_quoted_key_is_refused_without_a_line_number_rather_than_line_zero() {
        let reader = TemplateReader::parse("\"subject\" = 1\n").unwrap();
        let refusal = reader.string("subject").unwrap_err();
        assert_eq!(refusal, "subject must be a string");
    }

    #[test]
    fn a_bare_key_is_refused_with_its_line() {
        let reader = TemplateReader::parse("body = \"\"\nsubject = 1\n").unwrap();
        assert_eq!(
            reader.string("subject").unwrap_err(),
            "line 2: subject must be a string"
        );
    }
}
