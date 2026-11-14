use std::path::PathBuf;

use dam_application::{Config, ConfigError, Standing, TokenConfig, TokenTerms};
use dam_domain::Token;
use tempfile::TempDir;

use crate::lua_config::{LoadedConfig, LuaConfigFile};

pub(super) struct Folder {
    _dir: TempDir,
    root: PathBuf,
}

impl Folder {
    pub(super) fn new() -> Folder {
        let dir = tempfile::tempdir().unwrap();
        let root = dir.path().to_path_buf();
        Folder { _dir: dir, root }
    }

    pub(super) fn under_a_long_folder_name() -> Folder {
        let dir = tempfile::tempdir().unwrap();
        let root = dir
            .path()
            .join("a-folder-name-longer-than-lua-keeps".repeat(3));
        std::fs::create_dir(&root).unwrap();
        Folder { _dir: dir, root }
    }

    pub(super) fn with_config(self, text: &str) -> Folder {
        std::fs::write(self.path("config.lua"), text).unwrap();
        self
    }

    pub(super) fn with_secrets(self, text: &str) -> Folder {
        std::fs::write(self.path("secrets.lua"), text).unwrap();
        self
    }

    pub(super) fn marker(&self) -> std::path::PathBuf {
        self.path("called")
    }

    pub(super) fn path(&self, name: &str) -> std::path::PathBuf {
        self.root.join(name)
    }

    pub(super) fn loaded(&self) -> LoadedConfig {
        LuaConfigFile::new(self.path("config.lua")).load().unwrap()
    }

    pub(super) fn load(&self) -> Result<Config, ConfigError> {
        LuaConfigFile::new(self.path("config.lua"))
            .load()
            .map(|loaded| loaded.config)
    }

    pub(super) fn refusal(&self) -> String {
        self.load().unwrap_err().to_string()
    }
}

pub(super) fn config(text: &str) -> Folder {
    Folder::new().with_config(text)
}

pub(super) const ONE_REMOTE: &str = "return { remote = { todoist = { url = 'todoist::' } } }";

pub(super) fn secrets(text: &str) -> Folder {
    config(ONE_REMOTE).with_secrets(text)
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
