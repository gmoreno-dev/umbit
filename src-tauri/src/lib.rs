mod api;
mod commands;
mod config;
mod core;
mod easter;
mod models;
mod mpris;
mod paths;
mod update;

use std::sync::Arc;

use tauri::Manager;

pub fn run() {
    env_logger::Builder::from_env(
        env_logger::Env::default().default_filter_or("umbit=info,librespot=warn"),
    )
    .format_timestamp_millis()
    .init();

    tauri::Builder::default()
        .plugin(tauri_plugin_updater::Builder::new().build())
        .setup(|app| {
            // O librespot exige ser criado dentro de um runtime Tokio.
            let handle = app.handle().clone();
            let core = tauri::async_runtime::block_on(async move { core::Core::start(handle) })
                .map_err(std::io::Error::other)?;
            app.manage(core);
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::get_session,
            commands::login,
            commands::logout,
            commands::get_config,
            commands::set_theme,
            commands::set_client_id,
            commands::open_dashboard,
            commands::get_now_playing,
            commands::get_queue,
            commands::get_playlists,
            commands::get_playlist_tracks,
            commands::get_liked,
            commands::get_liked_uri,
            commands::get_saved_albums,
            commands::get_album_tracks,
            commands::search,
            commands::play_context,
            commands::play_tracks,
            commands::play_egg,
            commands::get_egg_tracks,
            commands::play_pause,
            commands::next_track,
            commands::prev_track,
            commands::seek,
            commands::seek_relative,
            commands::set_volume,
            commands::get_cover,
            commands::check_update,
            commands::install_update,
        ])
        .build(tauri::generate_context!())
        .expect("erro ao iniciar o Umbit")
        .run(|app, event| {
            if let tauri::RunEvent::Exit = event {
                if let Some(core) = app.try_state::<Arc<core::Core>>() {
                    core.shutdown();
                }
            }
        });
}
