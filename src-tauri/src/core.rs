//! O motor: sessão do Spotify (librespot), player, Connect, biblioteca,
//! busca, capas e o estado espelhado que a interface consome.

use std::{
    fs,
    num::NonZeroUsize,
    path::PathBuf,
    sync::{Arc, Mutex},
    time::{SystemTime, UNIX_EPOCH},
};

use librespot::{
    connect::{ConnectConfig, LoadRequest, LoadRequestOptions, PlayingTrack, Spirc},
    core::{
        authentication::Credentials,
        cache::Cache,
        config::{DeviceType, SessionConfig},
        Session, SpotifyUri,
    },
    metadata::audio::{AudioItem, UniqueFields},
    oauth::OAuthClientBuilder,
    playback::{
        audio_backend,
        config::{AudioFormat, Bitrate, PlayerConfig},
        mixer::{self, Mixer, MixerConfig},
        player::{Player, PlayerEvent, PlayerEventChannel},
    },
};
use lru::LruCache;
use tauri::{AppHandle, Emitter, Manager};
use tokio::sync::mpsc;

use crate::{
    api::{self, Api, TokenSource, OAUTH_REDIRECT, WEB_SCOPES},
    config::Config,
    easter::{self, EggTracks},
    models::{Album, ClientConfig, ErrorEvent, NowPlaying, Page, Playlist, QueueView, SearchResults, SessionInfo, Track},
    mpris::{self, MprisCommand, MprisUpdate},
    paths,
};

const OAUTH_SCOPES: &[&str] = &[
    "app-remote-control",
    "playlist-read",
    "playlist-read-collaborative",
    "playlist-read-private",
    "streaming",
    "user-library-read",
    "user-modify-playback-state",
    "user-read-currently-playing",
    "user-read-email",
    "user-read-playback-position",
    "user-read-playback-state",
    "user-read-private",
    "user-top-read",
];

/// Página que o navegador mostra depois de autorizar.
const OAUTH_PAGE: &str = r#"<!doctype html><html lang="pt-BR"><head><meta charset="utf-8"><title>umbit</title>
<style>body{margin:0;background:#f2efe6;color:#141210;font-family:'Courier New',monospace;display:flex;align-items:center;justify-content:center;height:100vh}
.b{border:2px solid #141210;box-shadow:6px 6px 0 #141210;padding:28px 32px;max-width:360px}
h1{font-size:22px;margin:0 0 10px 0;letter-spacing:.02em}p{margin:0;font-size:13px;opacity:.75;line-height:1.5}</style></head>
<body><div class="b"><h1>umbit</h1><p>pode voltar para o app. esta aba já pode ser fechada.</p></div></body></html>"#;

struct QueueMirror {
    name: Option<String>,
    uri: Option<String>,
    tracks: Vec<Track>,
    index: Option<usize>,
    note: Option<String>,
}

impl QueueMirror {
    fn empty() -> Self {
        Self { name: None, uri: None, tracks: Vec::new(), index: None, note: None }
    }

    fn view(&self, current: Option<Track>) -> QueueView {
        let upcoming = match self.index {
            Some(i) => self.tracks.iter().skip(i + 1).take(60).cloned().collect(),
            None => Vec::new(),
        };
        QueueView {
            context_name: self.name.clone(),
            context_uri: self.uri.clone(),
            current,
            current_index: self.index.map(|i| i as u32),
            upcoming,
            total: self.tracks.len() as u32,
            note: self.note.clone(),
        }
    }
}

pub struct Core {
    app: AppHandle,
    cache: Cache,
    session_cfg: SessionConfig,
    session: Mutex<Session>,
    player: Arc<Player>,
    mixer: Arc<dyn Mixer>,
    spirc: Mutex<Option<Spirc>>,
    config: Mutex<Config>,
    http: reqwest::Client,
    api: Mutex<Option<Api>>,
    now: Mutex<NowPlaying>,
    queue: Mutex<QueueMirror>,
    info: Mutex<SessionInfo>,
    covers: Mutex<LruCache<String, Arc<Vec<u8>>>>,
    egg: Mutex<Option<EggTracks>>,
    tokens: Mutex<Option<Arc<TokenSource>>>,
    mpris: mpsc::UnboundedSender<MprisUpdate>,
    login_lock: tokio::sync::Mutex<()>,
}

fn now_ms() -> u64 {
    SystemTime::now().duration_since(UNIX_EPOCH).map(|d| d.as_millis() as u64).unwrap_or(0)
}

fn pct_of(volume: u16) -> u8 {
    ((volume as f64 / u16::MAX as f64) * 100.0).round() as u8
}

fn volume_of(pct: u8) -> u16 {
    ((pct.min(100) as f64 / 100.0) * u16::MAX as f64).round() as u16
}

/// Converte a faixa do protocolo interno para o modelo da interface.
fn track_from_metadata(t: &librespot::metadata::Track) -> Track {
    let mut covers: Vec<(i32, String)> = t
        .album
        .covers
        .iter()
        .filter_map(|img| Some((img.width, format!("https://i.scdn.co/image/{}", img.id.to_base16().ok()?))))
        .collect();
    covers.sort_by_key(|(w, _)| *w);
    let cover = covers
        .iter()
        .find(|(w, _)| *w >= 250)
        .or_else(|| covers.last())
        .map(|(_, u)| u.clone());
    Track {
        uri: t.id.to_uri().unwrap_or_default(),
        name: t.name.clone(),
        artists: t.artists.iter().map(|a| a.name.clone()).collect(),
        album: t.album.name.clone(),
        duration_ms: t.duration.max(0) as u32,
        cover,
        extra: None,
    }
}

fn track_from_item(item: &AudioItem) -> Track {
    let uri = item.track_id.to_uri().unwrap_or_else(|_| item.uri.clone());
    let (artists, album) = match &item.unique_fields {
        UniqueFields::Track { artists, album, .. } => {
            (artists.iter().map(|a| a.name.clone()).collect(), album.clone())
        }
        UniqueFields::Episode { show_name, .. } => (vec![show_name.clone()], String::new()),
        UniqueFields::Local { artists, album, .. } => (
            artists.clone().map(|a| vec![a]).unwrap_or_default(),
            album.clone().unwrap_or_default(),
        ),
    };
    // `covers` vem ordenada da maior para a menor.
    let cover = item
        .covers
        .iter()
        .rev()
        .find(|c| c.width >= 250)
        .or_else(|| item.covers.first())
        .map(|c| c.url.clone());
    Track { uri, name: item.name.clone(), artists, album, duration_ms: item.duration_ms, cover, extra: None }
}

impl Core {
    pub fn start(app: AppHandle) -> Result<Arc<Self>, String> {
        let config = Config::load();
        let cache = Cache::new(
            Some(paths::credentials_dir()),
            Some(paths::volume_dir()),
            None::<PathBuf>,
            None,
        )
        .map_err(|e| format!("cache: {e}"))?;

        let mut session_cfg = SessionConfig::default();
        session_cfg.device_id = config.device_id.clone();
        let session = Session::new(session_cfg.clone(), Some(cache.clone()));

        let mixer_fn = mixer::find(Some("softvol")).ok_or("sem mixer de software")?;
        let mixer = mixer_fn(MixerConfig::default()).map_err(|e| format!("mixer: {e}"))?;
        let initial_volume = cache.volume().unwrap_or(volume_of(64));
        mixer.set_volume(initial_volume);

        let backend = audio_backend::find(None).ok_or("sem backend de áudio compilado")?;
        let player_cfg = PlayerConfig {
            bitrate: match config.bitrate {
                96 => Bitrate::Bitrate96,
                320 => Bitrate::Bitrate320,
                _ => Bitrate::Bitrate160,
            },
            gapless: true,
            ..PlayerConfig::default()
        };
        let player = Player::new(player_cfg, session.clone(), mixer.get_soft_volume(), move || {
            backend(None, AudioFormat::default())
        });

        let http = reqwest::Client::builder()
            .user_agent(format!("umbit/{}", env!("CARGO_PKG_VERSION")))
            .build()
            .map_err(|e| format!("http: {e}"))?;

        let (cmd_tx, cmd_rx) = mpsc::unbounded_channel::<MprisCommand>();
        let mpris_tx = mpris::start(cmd_tx);

        let now = NowPlaying { volume: pct_of(initial_volume), at_ms: now_ms(), ..Default::default() };

        let core = Arc::new(Core {
            app,
            cache: cache.clone(),
            session_cfg,
            session: Mutex::new(session),
            player: player.clone(),
            mixer,
            spirc: Mutex::new(None),
            config: Mutex::new(config),
            http,
            api: Mutex::new(None),
            now: Mutex::new(now),
            queue: Mutex::new(QueueMirror::empty()),
            info: Mutex::new(SessionInfo::default()),
            covers: Mutex::new(LruCache::new(NonZeroUsize::new(48).unwrap())),
            egg: Mutex::new(None),
            tokens: Mutex::new(None),
            mpris: mpris_tx,
            login_lock: tokio::sync::Mutex::new(()),
        });

        let events = player.get_player_event_channel();
        tauri::async_runtime::spawn(core.clone().event_loop(events));
        tauri::async_runtime::spawn(core.clone().mpris_loop(cmd_rx));

        if let Some(creds) = cache.credentials() {
            let c = core.clone();
            tauri::async_runtime::spawn(async move {
                c.connect_with(creds, true, None).await;
            });
        }
        Ok(core)
    }

    // ---------- estado e eventos ----------

    fn emit<T: serde::Serialize + Clone>(&self, name: &str, payload: T) {
        if let Err(e) = self.app.emit(name, payload) {
            log::warn!("emit {name}: {e}");
        }
    }

    fn emit_error(&self, message: impl Into<String>) {
        let message = message.into();
        log::warn!("{message}");
        self.emit("error", ErrorEvent { message });
    }

    fn emit_now(&self) {
        let now = self.now.lock().unwrap().clone();
        let _ = self.mpris.send(MprisUpdate::Now(now.clone()));
        self.emit("now_playing", now);
    }

    fn emit_queue(&self) {
        let current = self.now.lock().unwrap().track.clone();
        let view = self.queue.lock().unwrap().view(current);
        self.emit("queue", view);
    }

    fn emit_session(&self) {
        let info = self.info.lock().unwrap().clone();
        self.emit("session", info);
    }

    fn update_info(&self, f: impl FnOnce(&mut SessionInfo)) {
        {
            let mut info = self.info.lock().unwrap();
            f(&mut info);
            let cfg = self.config.lock().unwrap();
            info.easter_egg = easter::enabled(&cfg.easter_egg, info.username.as_deref());
            info.birthday = info.easter_egg && easter::is_birthday();
        }
        self.emit_session();
    }

    pub fn session_info(&self) -> SessionInfo {
        self.info.lock().unwrap().clone()
    }

    pub fn now_playing(&self) -> NowPlaying {
        self.now.lock().unwrap().clone()
    }

    pub fn queue_view(&self) -> QueueView {
        let current = self.now.lock().unwrap().track.clone();
        self.queue.lock().unwrap().view(current)
    }

    pub fn client_config(&self) -> ClientConfig {
        let cfg = self.config.lock().unwrap();
        ClientConfig {
            theme: cfg.theme.clone(),
            device_name: cfg.device_name.clone(),
            egg_theme_ink: cfg.easter_egg.theme_ink.clone(),
            egg_theme_paper: cfg.easter_egg.theme_paper.clone(),
            version: env!("CARGO_PKG_VERSION").to_string(),
        }
    }

    pub fn set_theme(&self, theme: &str) {
        let mut cfg = self.config.lock().unwrap();
        cfg.theme = theme.to_string();
        cfg.save();
    }

    fn egg_enabled(&self) -> bool {
        self.info.lock().unwrap().easter_egg
    }

    async fn event_loop(self: Arc<Self>, mut rx: PlayerEventChannel) {
        while let Some(ev) = rx.recv().await {
            let mut changed = true;
            let mut track_changed: Option<Track> = None;
            let mut seeked: Option<u32> = None;
            {
                let mut now = self.now.lock().unwrap();
                match ev {
                    PlayerEvent::TrackChanged { audio_item } => {
                        let t = track_from_item(&audio_item);
                        now.track = Some(t.clone());
                        now.loading = false;
                        track_changed = Some(t);
                    }
                    PlayerEvent::Loading { position_ms, .. } => {
                        now.loading = true;
                        now.position_ms = position_ms;
                        now.at_ms = now_ms();
                    }
                    PlayerEvent::Playing { position_ms, .. } => {
                        now.playing = true;
                        now.loading = false;
                        now.position_ms = position_ms;
                        now.at_ms = now_ms();
                    }
                    PlayerEvent::Paused { position_ms, .. } => {
                        now.playing = false;
                        now.loading = false;
                        now.position_ms = position_ms;
                        now.at_ms = now_ms();
                    }
                    PlayerEvent::Stopped { .. } => {
                        now.playing = false;
                        now.loading = false;
                        now.position_ms = 0;
                        now.at_ms = now_ms();
                    }
                    PlayerEvent::Seeked { position_ms, .. }
                    | PlayerEvent::PositionCorrection { position_ms, .. }
                    | PlayerEvent::PositionChanged { position_ms, .. } => {
                        now.position_ms = position_ms;
                        now.at_ms = now_ms();
                        seeked = Some(position_ms);
                    }
                    PlayerEvent::VolumeChanged { volume } => now.volume = pct_of(volume),
                    PlayerEvent::ShuffleChanged { shuffle } => now.shuffle = shuffle,
                    PlayerEvent::RepeatChanged { context, .. } => now.repeat = context,
                    PlayerEvent::SessionConnected { .. } => now.active = true,
                    PlayerEvent::SessionDisconnected { .. } => now.active = false,
                    PlayerEvent::Unavailable { .. } => {
                        drop(now);
                        self.emit_error("essa faixa não está disponível na sua região");
                        continue;
                    }
                    _ => changed = false,
                }
            }
            if let Some(t) = track_changed {
                self.on_track_changed(&t);
            }
            if let Some(ms) = seeked {
                let _ = self.mpris.send(MprisUpdate::Seeked(ms));
            }
            if changed {
                self.emit_now();
            }
        }
    }

    fn on_track_changed(&self, track: &Track) {
        let mut egg_track = false;
        let mut extra: Option<String> = None;
        {
            let mut q = self.queue.lock().unwrap();
            let found = q.tracks.iter().position(|t| t.uri == track.uri);
            match found {
                Some(i) => {
                    q.index = Some(i);
                    extra = q.tracks[i].extra.clone();
                }
                None => {
                    // Outro dispositivo trocou o contexto: o espelho não vale mais.
                    *q = QueueMirror::empty();
                }
            }
            if let Some(egg) = self.egg.lock().unwrap().as_ref() {
                egg_track = egg.ours.uri == track.uri || egg.hers.uri == track.uri;
            }
        }
        {
            let mut now = self.now.lock().unwrap();
            if now.egg && !egg_track {
                now.egg = false;
            }
            // A marcação ("egg-ours"/"egg-next") é o que a interface usa para os temas automáticos.
            if let Some(t) = now.track.as_mut() {
                t.extra = extra;
            }
        }
        self.emit_queue();
    }

    // ---------- sessão ----------

    async fn connect_with(self: Arc<Self>, creds: Credentials, from_cache: bool, oauth: Option<(librespot::oauth::OAuthToken, String)>) {
        let _guard = self.login_lock.lock().await;
        self.update_info(|i| {
            i.connecting = true;
            i.error = None;
        });

        let session = {
            let mut s = self.session.lock().unwrap();
            if s.is_invalid() {
                *s = Session::new(self.session_cfg.clone(), Some(self.cache.clone()));
                self.player.set_session(s.clone());
            }
            s.clone()
        };

        let connect_cfg = {
            let cfg = self.config.lock().unwrap();
            ConnectConfig {
                name: cfg.device_name.clone(),
                device_type: DeviceType::Computer,
                initial_volume: self.cache.volume().unwrap_or(volume_of(64)),
                ..ConnectConfig::default()
            }
        };

        match Spirc::new(connect_cfg, session.clone(), creds, self.player.clone(), self.mixer.clone()).await {
            Ok((spirc, task)) => {
                tauri::async_runtime::spawn(task);
                *self.spirc.lock().unwrap() = Some(spirc);
                let tokens = Arc::new(TokenSource::load(session.clone(), self.session_cfg.client_id.clone()));
                if let Some((t, cid)) = oauth {
                    tokens.store(&t, &cid);
                }
                *self.tokens.lock().unwrap() = Some(tokens.clone());
                let api = Api::new(self.http.clone(), tokens);
                *self.api.lock().unwrap() = Some(api.clone());
                let profile = api.me().await.ok();
                let mut username = session.username();
                if username.is_empty() || username == "UNKNOWN" {
                    username = profile.as_ref().map(|p| p.id.clone()).unwrap_or_default();
                }
                let display_name = profile.and_then(|p| p.display_name);
                self.update_info(|i| {
                    i.logged_in = true;
                    i.connecting = false;
                    i.username = Some(username.clone());
                    i.display_name = display_name;
                });
                log::info!("conectado como {username}");
                if self.egg_enabled() {
                    let c = self.clone();
                    tauri::async_runtime::spawn(async move {
                        let _ = c.egg_tracks().await;
                    });
                }
            }
            Err(e) => {
                let msg = format!("não deu para entrar: {e}");
                log::error!("{msg}");
                // A sessão fica suja depois de uma falha (o Connect já registrou
                // manipuladores nela). Derruba para a próxima tentativa nascer limpa.
                if !session.is_invalid() {
                    session.shutdown();
                }
                if from_cache {
                    // Credencial guardada não serve mais; pede login de novo.
                    let _ = fs::remove_file(paths::credentials_dir().join("credentials.json"));
                }
                self.update_info(|i| {
                    i.logged_in = false;
                    i.connecting = false;
                    i.error = Some(msg);
                });
            }
        }
    }

    async fn oauth(&self, client_id: &str, scopes: &[&str]) -> Result<librespot::oauth::OAuthToken, String> {
        let client = OAuthClientBuilder::new(client_id, OAUTH_REDIRECT, scopes.to_vec())
            .open_in_browser()
            .with_custom_message(OAUTH_PAGE)
            .build()
            .map_err(|e| format!("oauth: {e}"))?;
        client
            .get_access_token_async()
            .await
            .map_err(|e| format!("autorização não concluída: {e}"))
    }

    pub async fn login(self: Arc<Self>) -> Result<(), String> {
        if self.info.lock().unwrap().logged_in {
            return Ok(());
        }
        let own_id = self.config.lock().unwrap().client_id.trim().to_string();
        let shared_id = self.session_cfg.client_id.clone();
        self.update_info(|i| {
            i.connecting = true;
            i.error = None;
        });

        let fail = |me: &Self, msg: String| {
            me.update_info(|i| {
                i.connecting = false;
                i.error = Some(msg.clone());
            });
            msg
        };

        if own_id.is_empty() {
            // Só o client id compartilhado: um token para tudo.
            let t = match self.oauth(&shared_id, OAUTH_SCOPES).await {
                Ok(t) => t,
                Err(e) => return Err(fail(&self, e)),
            };
            let creds = Credentials::with_access_token(t.access_token.clone());
            self.clone().connect_with(creds, false, Some((t, shared_id))).await;
        } else {
            // 1. Token do seu app para a Web API (e, se o Spotify aceitar, para a sessão também).
            let web = match self.oauth(&own_id, WEB_SCOPES).await {
                Ok(t) => t,
                Err(e) => return Err(fail(&self, e)),
            };
            let creds = Credentials::with_access_token(web.access_token.clone());
            self.clone().connect_with(creds, false, Some((web.clone(), own_id.clone()))).await;
            if !self.session_info().logged_in {
                // 2. A sessão não aceitou o token do seu app: autoriza de novo com o id do librespot.
                log::warn!("sessão recusou o token do client id próprio; tentando o compartilhado");
                let t = match self.oauth(&shared_id, OAUTH_SCOPES).await {
                    Ok(t) => t,
                    Err(e) => return Err(fail(&self, e)),
                };
                let creds = Credentials::with_access_token(t.access_token.clone());
                // Guarda o token da Web API do seu app, não o do librespot.
                self.clone().connect_with(creds, false, Some((web, own_id))).await;
            }
        }
        let info = self.session_info();
        if info.logged_in {
            Ok(())
        } else {
            Err(info.error.unwrap_or_else(|| "não deu para entrar".into()))
        }
    }

    pub fn logout(&self) {
        if let Some(s) = self.spirc.lock().unwrap().take() {
            let _ = s.shutdown();
        }
        {
            let s = self.session.lock().unwrap();
            if !s.is_invalid() {
                s.shutdown();
            }
        }
        let _ = fs::remove_file(paths::credentials_dir().join("credentials.json"));
        TokenSource::clear();
        *self.api.lock().unwrap() = None;
        *self.tokens.lock().unwrap() = None;
        *self.egg.lock().unwrap() = None;
        {
            let mut now = self.now.lock().unwrap();
            let volume = now.volume;
            *now = NowPlaying { volume, at_ms: now_ms(), ..Default::default() };
        }
        *self.queue.lock().unwrap() = QueueMirror::empty();
        self.update_info(|i| *i = SessionInfo::default());
        self.emit_now();
        self.emit_queue();
    }

    pub fn shutdown(&self) {
        if let Some(s) = self.spirc.lock().unwrap().take() {
            let _ = s.shutdown();
        }
        let s = self.session.lock().unwrap();
        if !s.is_invalid() {
            s.shutdown();
        }
    }

    fn api(&self) -> Result<Api, String> {
        self.api.lock().unwrap().clone().ok_or_else(|| "você ainda não entrou".to_string())
    }

    fn with_spirc<R>(&self, f: impl FnOnce(&Spirc) -> Result<R, librespot::core::Error>) -> Result<R, String> {
        let guard = self.spirc.lock().unwrap();
        match guard.as_ref() {
            Some(s) => f(s).map_err(|e| format!("player: {e}")),
            None => Err("você ainda não entrou".into()),
        }
    }

    // ---------- reprodução ----------

    pub fn play_context(
        &self,
        uri: String,
        index: Option<u32>,
        track_uri: Option<String>,
        name: Option<String>,
        tracks: Vec<Track>,
    ) -> Result<(), String> {
        let playing_track = track_uri.clone().map(PlayingTrack::Uri).or(index.map(PlayingTrack::Index));
        self.with_spirc(|s| {
            let _ = s.activate();
            s.load(LoadRequest::from_context_uri(
                uri.clone(),
                LoadRequestOptions { start_playing: true, playing_track, ..LoadRequestOptions::default() },
            ))
        })?;
        {
            let mut q = self.queue.lock().unwrap();
            // Pular de dentro da própria fila manda a lista vazia: mantém a que já temos.
            let tracks = if tracks.is_empty() && q.uri.as_deref() == Some(uri.as_str()) {
                std::mem::take(&mut q.tracks)
            } else {
                tracks
            };
            let start = track_uri
                .and_then(|u| tracks.iter().position(|t| t.uri == u))
                .or(index.map(|i| i as usize));
            let name = name.or_else(|| q.name.clone());
            *q = QueueMirror { name, uri: Some(uri), tracks, index: start, note: None };
        }
        self.now.lock().unwrap().egg = false;
        self.emit_queue();
        Ok(())
    }

    pub fn play_tracks(&self, tracks: Vec<Track>, name: Option<String>, note: Option<String>, egg: bool) -> Result<(), String> {
        if tracks.is_empty() {
            return Err("nada para tocar".into());
        }
        let uris: Vec<String> = tracks.iter().map(|t| t.uri.clone()).collect();
        self.with_spirc(|s| {
            let _ = s.activate();
            s.load(LoadRequest::from_tracks(
                uris,
                LoadRequestOptions { start_playing: true, ..LoadRequestOptions::default() },
            ))
        })?;
        *self.queue.lock().unwrap() = QueueMirror { name, uri: None, tracks, index: Some(0), note };
        self.now.lock().unwrap().egg = egg;
        self.emit_queue();
        self.emit_now();
        Ok(())
    }

    pub fn play_pause(&self) -> Result<(), String> {
        self.with_spirc(|s| s.play_pause())
    }

    pub fn play(&self) -> Result<(), String> {
        self.with_spirc(|s| s.play())
    }

    pub fn pause(&self) -> Result<(), String> {
        self.with_spirc(|s| s.pause())
    }

    pub fn next(&self) -> Result<(), String> {
        self.with_spirc(|s| s.next())
    }

    pub fn prev(&self) -> Result<(), String> {
        self.with_spirc(|s| s.prev())
    }

    pub fn seek(&self, position_ms: u32) -> Result<(), String> {
        self.with_spirc(|s| s.set_position_ms(position_ms))?;
        {
            let mut now = self.now.lock().unwrap();
            now.position_ms = position_ms;
            now.at_ms = now_ms();
        }
        self.emit_now();
        Ok(())
    }

    pub fn seek_relative(&self, delta_ms: i64) -> Result<(), String> {
        let (pos, dur) = {
            let now = self.now.lock().unwrap();
            let elapsed = if now.playing { now_ms().saturating_sub(now.at_ms) } else { 0 };
            let pos = now.position_ms as i64 + elapsed as i64;
            (pos, now.track.as_ref().map(|t| t.duration_ms as i64).unwrap_or(i64::MAX))
        };
        let target = (pos + delta_ms).clamp(0, dur.saturating_sub(1000).max(0));
        self.seek(target as u32)
    }

    pub fn set_volume(&self, pct: u8) -> Result<(), String> {
        let v = volume_of(pct);
        self.with_spirc(|s| s.set_volume(v))?;
        self.now.lock().unwrap().volume = pct.min(100);
        self.emit_now();
        Ok(())
    }

    async fn mpris_loop(self: Arc<Self>, mut rx: mpsc::UnboundedReceiver<MprisCommand>) {
        while let Some(cmd) = rx.recv().await {
            let r = match cmd {
                MprisCommand::PlayPause => self.play_pause(),
                MprisCommand::Play => self.play(),
                MprisCommand::Pause => self.pause(),
                MprisCommand::Next => self.next(),
                MprisCommand::Prev => self.prev(),
                MprisCommand::SetPosition(ms) => self.seek(ms),
                MprisCommand::Seek(delta) => self.seek_relative(delta),
                MprisCommand::SetVolume(p) => self.set_volume(p),
                MprisCommand::Raise => {
                    if let Some(w) = self.app.get_webview_window("main") {
                        let _ = w.show();
                        let _ = w.unminimize();
                        let _ = w.set_focus();
                    }
                    Ok(())
                }
                MprisCommand::Quit => {
                    self.app.exit(0);
                    Ok(())
                }
            };
            if let Err(e) = r {
                log::debug!("mpris {cmd:?}: {e}");
            }
        }
    }

    // ---------- biblioteca e busca ----------

    pub async fn playlists(&self, offset: u32) -> Result<Page<Playlist>, String> {
        self.api()?.playlists(offset).await
    }

    pub async fn playlist_tracks(&self, uri: &str, offset: u32) -> Result<Page<Track>, String> {
        match self.api()?.playlist_tracks(api::id_of(uri), offset).await {
            Ok(page) if !page.items.is_empty() || page.total == 0 => Ok(page),
            Ok(_) => self.playlist_tracks_native(uri, offset).await,
            Err(e) => {
                log::info!("web api recusou a playlist ({e}); usando o caminho interno");
                self.playlist_tracks_native(uri, offset).await
            }
        }
    }

    /// Faixas de uma playlist pelo protocolo interno (spclient), que funciona
    /// para qualquer playlist, inclusive as que o usuário só segue.
    async fn playlist_tracks_native(&self, uri: &str, offset: u32) -> Result<Page<Track>, String> {
        use futures::StreamExt;
        use librespot::metadata::{Metadata, Playlist, Track as MdTrack};

        let session = self.session.lock().unwrap().clone();
        let id = SpotifyUri::from_uri(uri).map_err(|e| format!("uri: {e}"))?;
        let list = Playlist::get(&session, &id).await.map_err(|e| format!("playlist: {e}"))?;
        let uris: Vec<SpotifyUri> = list.tracks().cloned().collect();
        let total = uris.len() as u32;
        let slice: Vec<SpotifyUri> = uris.into_iter().skip(offset as usize).take(100).collect();
        let items: Vec<Track> = futures::stream::iter(slice)
            .map(|track_uri| {
                let session = session.clone();
                async move {
                    match MdTrack::get(&session, &track_uri).await {
                        Ok(t) => Some(track_from_metadata(&t)),
                        Err(e) => {
                            log::debug!("faixa {track_uri:?}: {e}");
                            None
                        }
                    }
                }
            })
            .buffered(8)
            .filter_map(|t| async move { t })
            .collect()
            .await;
        Ok(Page { items, total, offset })
    }

    pub async fn liked(&self, offset: u32) -> Result<Page<Track>, String> {
        self.api()?.liked(offset).await
    }

    /// URI do contexto "músicas curtidas" desta conta.
    pub fn liked_uri(&self) -> Option<String> {
        self.info.lock().unwrap().username.as_ref().map(|u| format!("spotify:user:{u}:collection"))
    }

    pub async fn saved_albums(&self, offset: u32) -> Result<Page<Album>, String> {
        self.api()?.saved_albums(offset).await
    }

    pub async fn album_tracks(&self, uri: &str) -> Result<Page<Track>, String> {
        self.api()?.album_tracks(api::id_of(uri)).await
    }

    pub async fn search(&self, q: &str) -> Result<SearchResults, String> {
        let api = self.api()?;
        let q = q.trim();
        if q.is_empty() {
            return Ok(SearchResults { query: String::new(), ..Default::default() });
        }
        let mut results = api.search(q).await?;
        if self.egg_enabled() && easter::matches(q) {
            if let Some(egg) = self.egg_tracks().await {
                results.tracks.insert(0, easter::decorate(&egg.hers));
            }
        }
        Ok(results)
    }

    // ---------- easter egg ----------

    async fn egg_tracks(&self) -> Option<EggTracks> {
        if let Some(e) = self.egg.lock().unwrap().clone() {
            return Some(e);
        }
        let api = self.api().ok()?;
        let hers = api.find_track(easter::HERS.0, easter::HERS.1).await.ok().flatten()?;
        let ours = api.find_track(easter::OURS.0, easter::OURS.1).await.ok().flatten()?;
        let egg = EggTracks { hers, ours };
        *self.egg.lock().unwrap() = Some(egg.clone());
        Some(egg)
    }

    /// As duas faixas, para a interface montar a linha fixa do aniversário.
    pub async fn egg_view(&self) -> Option<Vec<Track>> {
        if !self.egg_enabled() {
            return None;
        }
        let egg = self.egg_tracks().await?;
        Some(vec![easter::decorate(&egg.hers), egg.ours])
    }

    /// Toca a nossa primeiro, depois a dela.
    pub async fn play_egg(&self) -> Result<(), String> {
        if !self.egg_enabled() {
            return Err("nada aqui".into());
        }
        let egg = self.egg_tracks().await.ok_or("não achei as músicas no spotify")?;
        let mut hers = egg.hers.clone();
        hers.extra = Some("egg-next".into());
        let mut ours = egg.ours.clone();
        ours.extra = Some("egg-ours".into());
        self.play_tracks(vec![ours, hers], Some("ana lívia".into()), Some(easter::NOTE.into()), true)
    }

    // ---------- capas ----------

    pub async fn cover(&self, url: &str) -> Result<Arc<Vec<u8>>, String> {
        let ok_host = url.starts_with("https://i.scdn.co/")
            || url.starts_with("https://mosaic.scdn.co/")
            || url.starts_with("https://image-cdn-")
            || url.starts_with("https://lineup-images.scdn.co/");
        if !ok_host {
            return Err("origem de imagem não permitida".into());
        }
        if let Some(b) = self.covers.lock().unwrap().get(url) {
            return Ok(b.clone());
        }
        let bytes = self
            .http
            .get(url)
            .send()
            .await
            .map_err(|e| format!("capa: {e}"))?
            .bytes()
            .await
            .map_err(|e| format!("capa: {e}"))?;
        let arc = Arc::new(bytes.to_vec());
        self.covers.lock().unwrap().put(url.to_string(), arc.clone());
        Ok(arc)
    }
}
