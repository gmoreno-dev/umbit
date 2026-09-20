//! Comandos que a interface invoca. Camada fina sobre `Core`.

use std::sync::Arc;

use tauri::{AppHandle, State};

use crate::{
    core::Core,
    update::{self, UpdateInfo},
    models::{Album, ClientConfig, NowPlaying, Page, Playlist, QueueView, SearchResults, SessionInfo, Track},
};

type C<'a> = State<'a, Arc<Core>>;

#[tauri::command]
pub fn get_session(core: C) -> SessionInfo {
    core.session_info()
}

#[tauri::command]
pub async fn login(core: C<'_>) -> Result<(), String> {
    core.inner().clone().login().await
}

#[tauri::command]
pub fn logout(core: C) {
    core.logout()
}

#[tauri::command]
pub fn get_config(core: C) -> ClientConfig {
    core.client_config()
}

#[tauri::command]
pub fn set_theme(core: C, theme: String) {
    core.set_theme(&theme)
}

#[tauri::command]
pub fn set_client_id(core: C, client_id: String) -> Result<(), String> {
    core.set_client_id(&client_id)
}

/// Abre o painel de desenvolvedor do Spotify no navegador. Só essa URL.
#[tauri::command]
pub fn open_dashboard() -> Result<(), String> {
    open::that_detached("https://developer.spotify.com/dashboard").map_err(|e| format!("navegador: {e}"))
}

#[tauri::command]
pub fn get_now_playing(core: C) -> NowPlaying {
    core.now_playing()
}

#[tauri::command]
pub fn get_queue(core: C) -> QueueView {
    core.queue_view()
}

#[tauri::command]
pub async fn get_playlists(core: C<'_>, offset: Option<u32>) -> Result<Page<Playlist>, String> {
    core.playlists(offset.unwrap_or(0)).await
}

#[tauri::command]
pub async fn get_playlist_tracks(core: C<'_>, uri: String, offset: Option<u32>) -> Result<Page<Track>, String> {
    core.playlist_tracks(&uri, offset.unwrap_or(0)).await
}

#[tauri::command]
pub async fn get_liked(core: C<'_>, offset: Option<u32>) -> Result<Page<Track>, String> {
    core.liked(offset.unwrap_or(0)).await
}

#[tauri::command]
pub fn get_liked_uri(core: C) -> Option<String> {
    core.liked_uri()
}

#[tauri::command]
pub async fn get_saved_albums(core: C<'_>, offset: Option<u32>) -> Result<Page<Album>, String> {
    core.saved_albums(offset.unwrap_or(0)).await
}

#[tauri::command]
pub async fn get_album_tracks(core: C<'_>, uri: String) -> Result<Page<Track>, String> {
    core.album_tracks(&uri).await
}

#[tauri::command]
pub async fn search(core: C<'_>, query: String) -> Result<SearchResults, String> {
    core.search(&query).await
}

#[tauri::command]
pub fn play_context(
    core: C,
    uri: String,
    index: Option<u32>,
    track_uri: Option<String>,
    name: Option<String>,
    tracks: Option<Vec<Track>>,
) -> Result<(), String> {
    core.play_context(uri, index, track_uri, name, tracks.unwrap_or_default())
}

#[tauri::command]
pub fn play_tracks(core: C, tracks: Vec<Track>, name: Option<String>) -> Result<(), String> {
    core.play_tracks(tracks, name, None, false)
}

#[tauri::command]
pub async fn play_egg(core: C<'_>) -> Result<(), String> {
    core.play_egg().await
}

#[tauri::command]
pub async fn get_egg_tracks(core: C<'_>) -> Result<Option<Vec<Track>>, String> {
    Ok(core.egg_view().await)
}

#[tauri::command]
pub fn play_pause(core: C) -> Result<(), String> {
    core.play_pause()
}

#[tauri::command]
pub fn next_track(core: C) -> Result<(), String> {
    core.next()
}

#[tauri::command]
pub fn prev_track(core: C) -> Result<(), String> {
    core.prev()
}

#[tauri::command]
pub fn seek(core: C, position_ms: u32) -> Result<(), String> {
    core.seek(position_ms)
}

#[tauri::command]
pub fn seek_relative(core: C, delta_ms: i64) -> Result<(), String> {
    core.seek_relative(delta_ms)
}

#[tauri::command]
pub fn set_volume(core: C, volume: u8) -> Result<(), String> {
    core.set_volume(volume)
}

#[tauri::command]
pub fn set_shuffle(core: C, on: bool) -> Result<(), String> {
    core.set_shuffle(on)
}

#[tauri::command]
pub async fn get_cover(core: C<'_>, url: String) -> Result<tauri::ipc::Response, String> {
    let bytes = core.cover(&url).await?;
    Ok(tauri::ipc::Response::new(bytes.as_ref().clone()))
}

#[tauri::command]
pub async fn check_update(app: AppHandle, force: Option<bool>) -> Result<Option<UpdateInfo>, String> {
    update::check(&app, force.unwrap_or(false)).await
}

#[tauri::command]
pub async fn install_update(app: AppHandle) -> Result<(), String> {
    update::install(&app).await
}
