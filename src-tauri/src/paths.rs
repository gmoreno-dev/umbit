//! Onde o Umbit guarda as coisas no disco.

use std::path::PathBuf;

fn base(kind: Option<PathBuf>, fallback: &str) -> PathBuf {
    kind.unwrap_or_else(|| dirs::home_dir().unwrap_or_else(|| PathBuf::from(".")).join(fallback))
        .join("umbit")
}

/// `~/.config/umbit`
pub fn config_dir() -> PathBuf {
    base(dirs::config_dir(), ".config")
}

/// `~/.cache/umbit`
pub fn cache_dir() -> PathBuf {
    base(dirs::cache_dir(), ".cache")
}

pub fn config_file() -> PathBuf {
    config_dir().join("config.toml")
}

/// Onde o librespot guarda a credencial reutilizável (credentials.json).
pub fn credentials_dir() -> PathBuf {
    cache_dir().join("credentials")
}

pub fn volume_dir() -> PathBuf {
    cache_dir().join("volume")
}
