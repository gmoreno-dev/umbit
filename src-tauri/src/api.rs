//! Cliente da Web API do Spotify usando o token da própria sessão do
//! librespot (o mesmo caminho que o ncspot usa). Devolve os modelos leves
//! de `models.rs`, nunca o JSON cru.

use std::time::Duration;

use librespot::core::Session;
use serde_json::Value;

use crate::models::{Album, Artist, Page, Playlist, SearchResults, Track};

const BASE: &str = "https://api.spotify.com/v1";
const SCOPES: &str = "user-read-private,user-read-email,playlist-read-private,playlist-read-collaborative,user-library-read,user-top-read";

#[derive(Clone)]
pub struct Api {
    http: reqwest::Client,
    session: Session,
}

#[derive(Debug, Clone)]
pub struct Profile {
    pub id: String,
    pub display_name: Option<String>,
}

impl Api {
    pub fn new(http: reqwest::Client, session: Session) -> Self {
        Self { http, session }
    }

    async fn token(&self) -> Result<String, String> {
        let t = self
            .session
            .token_provider()
            .get_token(SCOPES)
            .await
            .map_err(|e| format!("token: {e}"))?;
        Ok(t.access_token)
    }

    async fn get(&self, path: &str) -> Result<Value, String> {
        let url = if path.starts_with("http") {
            path.to_string()
        } else {
            format!("{BASE}{path}")
        };
        let mut attempts = 0;
        loop {
            attempts += 1;
            let token = self.token().await?;
            let resp = self
                .http
                .get(&url)
                .bearer_auth(&token)
                .send()
                .await
                .map_err(|e| format!("rede: {e}"))?;
            let status = resp.status();
            if status.as_u16() == 429 && attempts < 3 {
                let wait = resp
                    .headers()
                    .get("retry-after")
                    .and_then(|v| v.to_str().ok())
                    .and_then(|s| s.parse::<u64>().ok())
                    .unwrap_or(1)
                    .min(10);
                tokio::time::sleep(Duration::from_secs(wait)).await;
                continue;
            }
            if !status.is_success() {
                let body = resp.text().await.unwrap_or_default();
                let msg = serde_json::from_str::<Value>(&body)
                    .ok()
                    .and_then(|v| v["error"]["message"].as_str().map(String::from))
                    .unwrap_or(body);
                return Err(format!("spotify {}: {}", status.as_u16(), msg.chars().take(200).collect::<String>()));
            }
            return resp.json::<Value>().await.map_err(|e| format!("json: {e}"));
        }
    }

    pub async fn me(&self) -> Result<Profile, String> {
        let v = self.get("/me").await?;
        Ok(Profile {
            id: str_of(&v["id"]).unwrap_or_default(),
            display_name: str_of(&v["display_name"]),
        })
    }

    pub async fn playlists(&self, offset: u32) -> Result<Page<Playlist>, String> {
        let v = self.get(&format!("/me/playlists?limit=50&offset={offset}")).await?;
        let items = arr(&v["items"])
            .iter()
            .filter_map(parse_playlist)
            .collect();
        Ok(Page { items, total: u32_of(&v["total"]), offset })
    }

    pub async fn playlist_tracks(&self, id: &str, offset: u32) -> Result<Page<Track>, String> {
        let fields = "total,items(track(uri,name,duration_ms,is_local,artists(name),album(name,images)))";
        let v = self
            .get(&format!("/playlists/{id}/tracks?limit=100&offset={offset}&fields={fields}"))
            .await?;
        let items = arr(&v["items"])
            .iter()
            .filter_map(|it| parse_track(&it["track"]))
            .collect();
        Ok(Page { items, total: u32_of(&v["total"]), offset })
    }

    pub async fn liked(&self, offset: u32) -> Result<Page<Track>, String> {
        let v = self.get(&format!("/me/tracks?limit=50&offset={offset}")).await?;
        let items = arr(&v["items"])
            .iter()
            .filter_map(|it| parse_track(&it["track"]))
            .collect();
        Ok(Page { items, total: u32_of(&v["total"]), offset })
    }

    pub async fn saved_albums(&self, offset: u32) -> Result<Page<Album>, String> {
        let v = self.get(&format!("/me/albums?limit=50&offset={offset}")).await?;
        let items = arr(&v["items"])
            .iter()
            .filter_map(|it| parse_album(&it["album"]))
            .collect();
        Ok(Page { items, total: u32_of(&v["total"]), offset })
    }

    pub async fn album_tracks(&self, id: &str) -> Result<Page<Track>, String> {
        let album = self.get(&format!("/albums/{id}")).await?;
        let album_name = str_of(&album["name"]).unwrap_or_default();
        let cover = pick_cover(&album["images"]);
        let mut items: Vec<Track> = arr(&album["tracks"]["items"])
            .iter()
            .filter_map(|t| parse_track_in_album(t, &album_name, cover.clone()))
            .collect();
        let mut next = str_of(&album["tracks"]["next"]);
        while let Some(url) = next {
            let v = self.get(&url).await?;
            items.extend(
                arr(&v["items"])
                    .iter()
                    .filter_map(|t| parse_track_in_album(t, &album_name, cover.clone())),
            );
            next = str_of(&v["next"]);
        }
        let total = items.len() as u32;
        Ok(Page { items, total, offset: 0 })
    }

    pub async fn search(&self, q: &str) -> Result<SearchResults, String> {
        let enc = urlencode(q);
        let v = self
            .get(&format!("/search?q={enc}&type=track,album,artist&limit=10&market=from_token"))
            .await?;
        Ok(SearchResults {
            query: q.to_string(),
            tracks: arr(&v["tracks"]["items"]).iter().filter_map(parse_track).collect(),
            albums: arr(&v["albums"]["items"]).iter().filter_map(parse_album).collect(),
            artists: arr(&v["artists"]["items"]).iter().filter_map(parse_artist).collect(),
        })
    }

    /// Busca uma faixa específica por nome e artista. Usada pelo easter egg.
    pub async fn find_track(&self, name: &str, artist: &str) -> Result<Option<Track>, String> {
        let q = urlencode(&format!("track:{name} artist:{artist}"));
        let v = self
            .get(&format!("/search?q={q}&type=track&limit=5&market=from_token"))
            .await?;
        let name_l = name.to_lowercase();
        let artist_l = artist.to_lowercase();
        let found = arr(&v["tracks"]["items"])
            .iter()
            .filter_map(parse_track)
            .find(|t| {
                t.name.to_lowercase().starts_with(&name_l)
                    && t.artists.iter().any(|a| a.to_lowercase() == artist_l)
            });
        Ok(found)
    }
}

// ---------- parsing ----------

fn arr(v: &Value) -> &[Value] {
    v.as_array().map(|a| a.as_slice()).unwrap_or(&[])
}

fn str_of(v: &Value) -> Option<String> {
    v.as_str().map(String::from)
}

fn u32_of(v: &Value) -> u32 {
    v.as_u64().unwrap_or(0) as u32
}

fn names(v: &Value) -> Vec<String> {
    arr(v).iter().filter_map(|a| str_of(&a["name"])).collect()
}

/// Escolhe a menor imagem com pelo menos 250 px; senão a maior que houver.
fn pick_cover(images: &Value) -> Option<String> {
    let mut imgs: Vec<(u64, String)> = arr(images)
        .iter()
        .filter_map(|i| Some((i["width"].as_u64().unwrap_or(0), str_of(&i["url"])?)))
        .collect();
    if imgs.is_empty() {
        return None;
    }
    imgs.sort_by_key(|(w, _)| *w);
    imgs.iter()
        .find(|(w, _)| *w >= 250)
        .or_else(|| imgs.last())
        .map(|(_, u)| u.clone())
}

fn parse_track(t: &Value) -> Option<Track> {
    let uri = str_of(&t["uri"])?;
    if t["is_local"].as_bool().unwrap_or(false) || !uri.starts_with("spotify:track:") {
        return None;
    }
    Some(Track {
        uri,
        name: str_of(&t["name"]).unwrap_or_default(),
        artists: names(&t["artists"]),
        album: str_of(&t["album"]["name"]).unwrap_or_default(),
        duration_ms: u32_of(&t["duration_ms"]),
        cover: pick_cover(&t["album"]["images"]),
        extra: None,
    })
}

fn parse_track_in_album(t: &Value, album: &str, cover: Option<String>) -> Option<Track> {
    let uri = str_of(&t["uri"])?;
    if !uri.starts_with("spotify:track:") {
        return None;
    }
    Some(Track {
        uri,
        name: str_of(&t["name"]).unwrap_or_default(),
        artists: names(&t["artists"]),
        album: album.to_string(),
        duration_ms: u32_of(&t["duration_ms"]),
        cover,
        extra: None,
    })
}

fn parse_playlist(p: &Value) -> Option<Playlist> {
    Some(Playlist {
        uri: str_of(&p["uri"])?,
        name: str_of(&p["name"]).unwrap_or_default(),
        owner: str_of(&p["owner"]["display_name"])
            .or_else(|| str_of(&p["owner"]["id"]))
            .unwrap_or_default(),
        track_count: u32_of(&p["tracks"]["total"]),
        cover: pick_cover(&p["images"]),
    })
}

fn parse_album(a: &Value) -> Option<Album> {
    Some(Album {
        uri: str_of(&a["uri"])?,
        name: str_of(&a["name"]).unwrap_or_default(),
        artists: names(&a["artists"]),
        year: str_of(&a["release_date"]).map(|d| d.chars().take(4).collect()),
        track_count: u32_of(&a["total_tracks"]),
        cover: pick_cover(&a["images"]),
    })
}

fn parse_artist(a: &Value) -> Option<Artist> {
    Some(Artist {
        uri: str_of(&a["uri"])?,
        name: str_of(&a["name"]).unwrap_or_default(),
        cover: pick_cover(&a["images"]),
    })
}

fn urlencode(s: &str) -> String {
    let mut out = String::with_capacity(s.len() * 3);
    for b in s.bytes() {
        match b {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' => out.push(b as char),
            b' ' => out.push_str("%20"),
            _ => out.push_str(&format!("%{b:02X}")),
        }
    }
    out
}

/// Extrai o id base62 de um URI `spotify:tipo:id`.
pub fn id_of(uri: &str) -> &str {
    uri.rsplit(':').next().unwrap_or(uri)
}
