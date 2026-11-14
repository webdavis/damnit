use std::path::PathBuf;

use super::sandbox::Sandbox;
use super::token_table::{TokenTable, list, text};

pub struct Remote {
    url: String,
    stale: Option<String>,
    deadline: Option<String>,
    api_token: Option<String>,
}

impl Remote {
    pub fn at(url: &str) -> Remote {
        Remote {
            url: url.to_string(),
            stale: None,
            deadline: None,
            api_token: None,
        }
    }

    pub fn fake() -> Remote {
        Remote::at("fake::").api_token("tok-123")
    }

    pub fn stale(self, after: &str) -> Remote {
        Remote {
            stale: Some(after.to_string()),
            ..self
        }
    }

    pub fn deadline(self, silence: &str) -> Remote {
        Remote {
            deadline: Some(silence.to_string()),
            ..self
        }
    }

    pub fn api_token(self, value: &str) -> Remote {
        Remote {
            api_token: Some(value.to_string()),
            ..self
        }
    }
}

#[derive(Default)]
pub(super) struct ConfigFiles {
    owner: Option<String>,
    remotes: Vec<(String, Remote)>,
    categories: Vec<String>,
    filters: Vec<String>,
    tokens: Vec<String>,
}

impl ConfigFiles {
    pub(super) fn with_the_fake_remote() -> ConfigFiles {
        ConfigFiles {
            remotes: vec![("fake".to_string(), Remote::fake())],
            ..ConfigFiles::default()
        }
    }

    fn config_lua(&self) -> String {
        let owner = self
            .owner
            .iter()
            .map(|o| format!("  owner = {},\n", text(o)));
        let remotes = self.remotes.iter().map(|(name, remote)| {
            let mut fields = vec![format!("url = {}", text(&remote.url))];
            fields.extend(remote.stale.iter().map(|s| format!("stale = {}", text(s))));
            fields.extend(
                remote
                    .deadline
                    .iter()
                    .map(|d| format!("deadline = {}", text(d))),
            );
            entry(name, &format!("{{ {} }}", fields.join(", ")))
        });
        format!(
            "return {{\n{}{}{}{}}}\n",
            owner.collect::<String>(),
            section("remote", remotes),
            section("category", self.categories.iter().cloned()),
            section("filter", self.filters.iter().cloned())
        )
    }

    fn secrets_lua(&self) -> String {
        let credentials = self.remotes.iter().filter_map(|(name, remote)| {
            let value = remote.api_token.as_ref()?;
            Some(entry(name, &format!("{{ api_token = {} }}", text(value))))
        });
        format!(
            "return {{\n  schema = 1,\n{}{}}}\n",
            section("remote", credentials),
            section("token", self.tokens.iter().cloned())
        )
    }
}

impl Sandbox {
    pub fn set_owner(&self, name: &str) {
        self.change_config(|files| files.owner = Some(name.to_string()));
    }

    pub fn add_remote(&self, name: &str, remote: Remote) {
        self.change_config(|files| {
            files.remotes.retain(|(held, _)| held != name);
            files.remotes.push((name.to_string(), remote));
        });
    }

    pub fn add_category(&self, name: &str, values: &[&str], exclusive: bool) {
        let table = format!("{{ values = {}, exclusive = {exclusive} }}", list(values));
        self.change_config(|files| files.categories.push(entry(name, &table)));
    }

    pub fn add_filter(&self, name: &str, query: &str) {
        let table = format!("{{ query = {} }}", text(query));
        self.change_config(|files| files.filters.push(entry(name, &table)));
    }

    pub fn add_token(&self, name: &str, token: TokenTable) {
        self.add_token_written_as(name, &token.lua());
    }

    pub fn add_token_written_as(&self, name: &str, lua_table: &str) {
        let named = entry(name, "");
        self.change_config(|files| {
            files.tokens.retain(|held| !held.starts_with(&named));
            files.tokens.push(entry(name, lua_table));
        });
    }

    pub fn config_path(&self) -> PathBuf {
        self.dir.path().join("config.lua")
    }

    pub fn secrets_path(&self) -> PathBuf {
        self.dir.path().join("secrets.lua")
    }

    pub fn write_config_lua(&self, text: &str) {
        std::fs::write(self.config_path(), text).unwrap();
    }

    pub fn write_secrets_lua(&self, text: &str) {
        std::fs::write(self.secrets_path(), text).unwrap();
    }

    pub(super) fn write_config_files(&self) {
        let files = self.files.borrow();
        self.write_config_lua(&files.config_lua());
        self.write_secrets_lua(&files.secrets_lua());
    }

    fn change_config(&self, change: impl FnOnce(&mut ConfigFiles)) {
        change(&mut self.files.borrow_mut());
        self.write_config_files();
    }
}

fn section(name: &str, entries: impl Iterator<Item = String>) -> String {
    let body: String = entries.map(|e| format!("    {e},\n")).collect();
    format!("  {name} = {{\n{body}  }},\n")
}

fn entry(name: &str, table: &str) -> String {
    format!("{name} = {table}")
}
