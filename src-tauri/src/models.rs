//! Tipos que atravessam a ponte entre o núcleo e a interface.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Track {
    pub uri: String,
    pub name: String,
    pub artists: Vec<String>,
    pub album: String,
    pub duration_ms: u32,
    /// URL da capa (pequena o bastante para a lista, ~300 px).
    pub cover: Option<String>,
    /// Marcações especiais. "egg" = linha do easter egg.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub extra: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Playlist {
    pub uri: String,
    pub name: String,
    pub owner: String,
    pub track_count: u32,
    pub cover: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Album {
    pub uri: String,
    pub name: String,
    pub artists: Vec<String>,
    pub year: Option<String>,
    pub track_count: u32,
    pub cover: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Artist {
    pub uri: String,
    pub name: String,
    pub cover: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Page<T> {
    pub items: Vec<T>,
    pub total: u32,
    pub offset: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct SearchResults {
    pub query: String,
    pub tracks: Vec<Track>,
    pub albums: Vec<Album>,
    pub artists: Vec<Artist>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct SessionInfo {
    pub logged_in: bool,
    pub connecting: bool,
    pub username: Option<String>,
    pub display_name: Option<String>,
    /// Easter egg ligado para esta conta.
    pub easter_egg: bool,
    /// Hoje é 18 de março e o easter egg está ligado.
    pub birthday: bool,
    pub error: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct NowPlaying {
    pub track: Option<Track>,
    pub playing: bool,
    pub loading: bool,
    pub position_ms: u32,
    /// Instante (unix ms) em que `position_ms` foi amostrado; a interface
    /// interpola a partir daqui em vez de receber um evento a cada meio segundo.
    pub at_ms: u64,
    /// 0 a 100.
    pub volume: u8,
    pub shuffle: bool,
    pub repeat: bool,
    /// Este dispositivo é o ativo no Spotify Connect.
    pub active: bool,
    /// A reprodução atual é a do easter egg (revelação de título).
    pub egg: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct QueueView {
    pub context_name: Option<String>,
    pub context_uri: Option<String>,
    pub current: Option<Track>,
    pub current_index: Option<u32>,
    pub upcoming: Vec<Track>,
    pub total: u32,
    pub note: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClientConfig {
    pub theme: String,
    pub device_name: String,
    pub egg_theme_ink: String,
    pub egg_theme_paper: String,
    pub version: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct ErrorEvent {
    pub message: String,
}
