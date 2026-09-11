//! Teclas de mídia no Linux via MPRIS (D-Bus). Roda numa thread própria
//! porque o `Player` do mpris-server não é `Send`. Fora do Linux é um
//! canal que só descarta as atualizações (o Windows entra depois, via SMTC).

#[cfg(not(target_os = "linux"))]
use tokio::sync::mpsc;

use crate::models::NowPlaying;

#[derive(Debug)]
pub enum MprisUpdate {
    Now(NowPlaying),
    Seeked(u32),
}

#[derive(Debug, Clone, Copy)]
pub enum MprisCommand {
    PlayPause,
    Play,
    Pause,
    Next,
    Prev,
    /// Posição absoluta em ms.
    SetPosition(u32),
    /// Deslocamento relativo em ms.
    Seek(i64),
    /// 0 a 100.
    SetVolume(u8),
    Raise,
    Quit,
}

#[cfg(target_os = "linux")]
pub use linux::start;

/// Sem MPRIS: drena o canal para o núcleo não acumular mensagens.
#[cfg(not(target_os = "linux"))]
pub fn start(_cmd_tx: mpsc::UnboundedSender<MprisCommand>) -> mpsc::UnboundedSender<MprisUpdate> {
    let (tx, mut rx) = mpsc::unbounded_channel::<MprisUpdate>();
    tauri::async_runtime::spawn(async move { while rx.recv().await.is_some() {} });
    tx
}

#[cfg(target_os = "linux")]
mod linux {
    use mpris_server::{Metadata, PlaybackStatus, Player, Time, TrackId};
    use tokio::sync::mpsc;

    use super::{MprisCommand, MprisUpdate};
    use crate::models::NowPlaying;

    pub fn start(cmd_tx: mpsc::UnboundedSender<MprisCommand>) -> mpsc::UnboundedSender<MprisUpdate> {
        let (tx, rx) = mpsc::unbounded_channel::<MprisUpdate>();
        std::thread::Builder::new()
            .name("umbit-mpris".into())
            .spawn(move || {
                let rt = match tokio::runtime::Builder::new_current_thread().enable_all().build() {
                    Ok(rt) => rt,
                    Err(e) => {
                        log::warn!("mpris: sem runtime: {e}");
                        return;
                    }
                };
                let local = tokio::task::LocalSet::new();
                local.block_on(&rt, serve(cmd_tx, rx));
            })
            .ok();
        tx
    }

    async fn serve(cmd_tx: mpsc::UnboundedSender<MprisCommand>, mut rx: mpsc::UnboundedReceiver<MprisUpdate>) {
        let player = match Player::builder("umbit")
            .identity("Umbit")
            .desktop_entry("umbit")
            .can_play(true)
            .can_pause(true)
            .can_go_next(true)
            .can_go_previous(true)
            .can_seek(true)
            .can_raise(true)
            .can_quit(true)
            .build()
            .await
        {
            Ok(p) => p,
            Err(e) => {
                log::warn!("mpris indisponível (sem D-Bus?): {e}");
                // Sem MPRIS o app continua; só drena o canal para não acumular.
                while rx.recv().await.is_some() {}
                return;
            }
        };

        macro_rules! wire {
            ($connect:ident, $cmd:expr) => {{
                let tx = cmd_tx.clone();
                player.$connect(move |_| {
                    let _ = tx.send($cmd);
                });
            }};
        }
        wire!(connect_play_pause, MprisCommand::PlayPause);
        wire!(connect_play, MprisCommand::Play);
        wire!(connect_pause, MprisCommand::Pause);
        wire!(connect_stop, MprisCommand::Pause);
        wire!(connect_next, MprisCommand::Next);
        wire!(connect_previous, MprisCommand::Prev);
        wire!(connect_raise, MprisCommand::Raise);
        wire!(connect_quit, MprisCommand::Quit);
        {
            let tx = cmd_tx.clone();
            player.connect_seek(move |_, offset| {
                let _ = tx.send(MprisCommand::Seek(offset.as_millis()));
            });
        }
        {
            let tx = cmd_tx.clone();
            player.connect_set_position(move |_, _id, pos| {
                let _ = tx.send(MprisCommand::SetPosition(pos.as_millis().max(0) as u32));
            });
        }
        {
            let tx = cmd_tx.clone();
            player.connect_set_volume(move |_, v| {
                let _ = tx.send(MprisCommand::SetVolume((v.clamp(0.0, 1.0) * 100.0).round() as u8));
            });
        }

        tokio::task::spawn_local(player.run());

        while let Some(update) = rx.recv().await {
            match update {
                MprisUpdate::Now(now) => apply(&player, &now).await,
                MprisUpdate::Seeked(ms) => {
                    let _ = player.seeked(Time::from_millis(ms as i64)).await;
                }
            }
        }
    }

    async fn apply(player: &Player, now: &NowPlaying) {
        let status = if now.track.is_none() {
            PlaybackStatus::Stopped
        } else if now.playing {
            PlaybackStatus::Playing
        } else {
            PlaybackStatus::Paused
        };
        let _ = player.set_playback_status(status).await;

        let md = match &now.track {
            Some(t) => {
                let id = t.uri.rsplit(':').next().unwrap_or("0");
                let mut b = Metadata::builder()
                    .title(t.name.clone())
                    .artist(t.artists.clone())
                    .album(t.album.clone())
                    .length(Time::from_millis(t.duration_ms as i64));
                if let Ok(tid) = TrackId::try_from(format!("/dev/gmoreno/umbit/track/{id}")) {
                    b = b.trackid(tid);
                }
                if let Some(c) = &t.cover {
                    b = b.art_url(c.clone());
                }
                b.build()
            }
            None => Metadata::new(),
        };
        let _ = player.set_metadata(md).await;
        player.set_position(Time::from_millis(now.position_ms as i64));
        let _ = player.set_volume(now.volume as f64 / 100.0).await;
    }
}
