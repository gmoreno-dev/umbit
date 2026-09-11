//! Configuração do usuário em `~/.config/umbit/config.toml`.

use serde::{Deserialize, Serialize};
use std::fs;

use crate::paths;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct Config {
    /// Tema visual: "papel", "ambar", "fosforo", "gameboy" ou "livia".
    pub theme: String,
    /// Nome que aparece na lista de dispositivos do Spotify Connect.
    pub device_name: String,
    /// Identificador estável do dispositivo (gerado na primeira execução).
    pub device_id: String,
    /// 96, 160 ou 320 kbps.
    pub bitrate: u16,
    /// Client id de um app criado por você em developer.spotify.com. A Web API
    /// limita por aplicação, e o id compartilhado do librespot vive no limite.
    /// Vazio = usa o compartilhado.
    pub client_id: String,
    pub easter_egg: EasterEggConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct EasterEggConfig {
    /// "auto" dispara só para as contas em `users`; "always" para qualquer
    /// conta; "never" desliga tudo.
    pub mode: String,
    /// Nomes de usuário do Spotify para os quais o easter egg dispara.
    pub users: Vec<String>,
    /// Cor da tinta do tema escondido.
    pub theme_ink: String,
    /// Cor do papel do tema escondido.
    pub theme_paper: String,
}

impl Default for EasterEggConfig {
    fn default() -> Self {
        Self {
            mode: "auto".into(),
            users: Vec::new(),
            theme_ink: "#d1345b".into(),
            theme_paper: "#fff3f5".into(),
        }
    }
}

impl Default for Config {
    fn default() -> Self {
        Self {
            theme: "papel".into(),
            device_name: "Umbit".into(),
            device_id: String::new(),
            bitrate: 160,
            client_id: String::new(),
            easter_egg: EasterEggConfig::default(),
        }
    }
}

impl Config {
    pub fn load() -> Self {
        let path = paths::config_file();
        let mut cfg = fs::read_to_string(&path)
            .ok()
            .and_then(|s| toml::from_str::<Config>(&s).ok())
            .unwrap_or_default();
        let mut dirty = false;
        if cfg.device_id.is_empty() {
            cfg.device_id = uuid_v4();
            dirty = true;
        }
        if !matches!(cfg.bitrate, 96 | 160 | 320) {
            cfg.bitrate = 160;
            dirty = true;
        }
        if dirty || !path.exists() {
            cfg.save();
        }
        cfg
    }

    pub fn save(&self) {
        let path = paths::config_file();
        if let Some(dir) = path.parent() {
            let _ = fs::create_dir_all(dir);
        }
        if let Ok(s) = toml::to_string_pretty(self) {
            let _ = fs::write(&path, s);
        }
    }
}

/// UUID v4 sem depender de mais um crate: 16 bytes aleatórios do sistema.
fn uuid_v4() -> String {
    let mut bytes = [0u8; 16];
    let seed = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_nanos())
        .unwrap_or(0);
    let mut x = seed as u64 ^ 0x9E37_79B9_7F4A_7C15 ^ (std::process::id() as u64) << 32;
    for b in bytes.iter_mut() {
        // xorshift64*
        x ^= x >> 12;
        x ^= x << 25;
        x ^= x >> 27;
        *b = (x.wrapping_mul(0x2545_F491_4F6C_DD1D) >> 56) as u8;
    }
    bytes[6] = (bytes[6] & 0x0f) | 0x40;
    bytes[8] = (bytes[8] & 0x3f) | 0x80;
    let h: Vec<String> = bytes.iter().map(|b| format!("{b:02x}")).collect();
    format!(
        "{}{}{}{}-{}{}-{}{}-{}{}-{}{}{}{}{}{}",
        h[0], h[1], h[2], h[3], h[4], h[5], h[6], h[7], h[8], h[9], h[10], h[11], h[12], h[13], h[14], h[15]
    )
}
