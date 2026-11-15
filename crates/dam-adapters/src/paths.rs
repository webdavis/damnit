use std::path::PathBuf;

pub fn default_config_path() -> PathBuf {
    config_home().join("dam").join("config.lua")
}

pub fn default_store_path() -> PathBuf {
    under("XDG_DATA_HOME", &[".local", "share"])
        .join("dam")
        .join("dam.db")
}

pub fn default_state_home() -> PathBuf {
    under("XDG_STATE_HOME", &[".local", "state"])
}

pub fn default_last_run_path() -> PathBuf {
    default_state_home().join("dam").join("sync-last-run.txt")
}

pub(crate) fn config_home() -> PathBuf {
    under("XDG_CONFIG_HOME", &[".config"])
}

fn under(variable: &str, below_home: &[&str]) -> PathBuf {
    std::env::var_os(variable)
        .map(PathBuf::from)
        .unwrap_or_else(|| {
            below_home
                .iter()
                .fold(home_dir(), |dir, part| dir.join(part))
        })
}

pub(crate) fn home_dir() -> PathBuf {
    std::env::var_os("HOME")
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("."))
}
