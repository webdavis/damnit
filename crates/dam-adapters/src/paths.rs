use std::path::PathBuf;

pub fn default_config_path() -> PathBuf {
    under("XDG_CONFIG_HOME", &[".config"])
        .join("dam")
        .join("config.lua")
}

pub fn default_store_path() -> PathBuf {
    under("XDG_DATA_HOME", &[".local", "share"])
        .join("dam")
        .join("dam.db")
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

fn home_dir() -> PathBuf {
    std::env::var_os("HOME")
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("."))
}
