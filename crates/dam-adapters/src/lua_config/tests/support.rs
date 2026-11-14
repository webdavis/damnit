use dam_application::{Config, ConfigError, Standing, TokenConfig, TokenTerms};
use dam_domain::Token;
use tempfile::TempDir;

use crate::lua_config::LuaConfigFile;

pub(super) struct Folder {
    dir: TempDir,
}

impl Folder {
    pub(super) fn new() -> Folder {
        Folder {
            dir: tempfile::tempdir().unwrap(),
        }
    }

    pub(super) fn with_config(self, text: &str) -> Folder {
        std::fs::write(self.dir.path().join("config.lua"), text).unwrap();
        self
    }

    pub(super) fn with_secrets(self, text: &str) -> Folder {
        std::fs::write(self.dir.path().join("secrets.lua"), text).unwrap();
        self
    }

    pub(super) fn marker(&self) -> std::path::PathBuf {
        self.path("called")
    }

    pub(super) fn path(&self, name: &str) -> std::path::PathBuf {
        self.dir.path().join(name)
    }

    pub(super) fn file(&self) -> LuaConfigFile {
        LuaConfigFile::new(self.dir.path().join("config.lua"))
    }

    pub(super) fn load(&self) -> Result<Config, ConfigError> {
        self.file().load()
    }

    pub(super) fn refusal(&self) -> String {
        self.load().unwrap_err().to_string()
    }
}

pub(super) fn config(text: &str) -> Folder {
    Folder::new().with_config(text)
}

pub(super) fn secrets(text: &str) -> Folder {
    config("return { remote = { todoist = { url = 'todoist::' } } }").with_secrets(text)
}

pub(super) fn value(fill: u8) -> String {
    Token::generate(&mut |b: &mut [u8]| b.fill(fill)).expose()
}

pub(super) fn tokens(text: &str) -> (Vec<TokenConfig>, Vec<String>) {
    let tokens = secrets(text).load().unwrap().tokens;
    (tokens.entries, tokens.warnings)
}

pub(super) fn token(body: &str) -> String {
    format!(
        "return {{ token = {{ tool = {{ value = {:?}, {body} }} }} }}",
        value(1)
    )
}

pub(super) fn only_terms(text: &str) -> TokenTerms {
    match &tokens(text).0[..] {
        [
            TokenConfig {
                standing: Standing::Usable(terms),
                ..
            },
        ] => terms.clone(),
        other => panic!("expected one usable token, got {other:?}"),
    }
}

pub(super) fn only_disabled_reason(text: &str) -> String {
    match &tokens(text).0[..] {
        [
            TokenConfig {
                standing: Standing::Disabled(why),
                ..
            },
        ] => why.clone(),
        other => panic!("expected one disabled token, got {other:?}"),
    }
}

pub(super) fn marking_credential(marker: &std::path::Path) -> String {
    format!(
        "return {{ remote = {{ todoist = {{ api_token = function() \
           io.open({marker:?}, 'w'):close() return 'abc' end }} }} }}"
    )
}
