//! Atualização automática: uma verificação na abertura, no máximo uma vez a
//! cada 20 horas, com assinatura verificada pelo plugin do Tauri. Instala
//! sozinho onde dá (Windows e AppImage); nos pacotes que exigem root (.deb e
//! o binário em /opt) só avisa e mostra o comando.

use std::{fs, time::Duration};

use serde::Serialize;
use tauri::AppHandle;
use tauri_plugin_updater::UpdaterExt;

use crate::paths;

#[derive(Debug, Clone, Serialize)]
pub struct UpdateInfo {
    pub version: String,
    pub notes: Option<String>,
    /// O app consegue se atualizar sozinho nesta instalação.
    pub auto: bool,
}

pub fn auto_supported() -> bool {
    cfg!(target_os = "windows") || std::env::var_os("APPIMAGE").is_some()
}

fn stamp() -> std::path::PathBuf {
    paths::cache_dir().join("last-update-check")
}

fn checked_recently() -> bool {
    fs::metadata(stamp())
        .and_then(|m| m.modified())
        .ok()
        .and_then(|t| t.elapsed().ok())
        .map(|age| age < Duration::from_secs(20 * 60 * 60))
        .unwrap_or(false)
}

pub async fn check(app: &AppHandle, force: bool) -> Result<Option<UpdateInfo>, String> {
    if !force && checked_recently() {
        return Ok(None);
    }
    let _ = fs::create_dir_all(paths::cache_dir());
    let _ = fs::write(stamp(), b"");
    let updater = app
        .updater_builder()
        .timeout(Duration::from_secs(8))
        .build()
        .map_err(|e| format!("atualizador: {e}"))?;
    match updater.check().await {
        Ok(Some(u)) => Ok(Some(UpdateInfo { version: u.version.clone(), notes: u.body.clone(), auto: auto_supported() })),
        Ok(None) => Ok(None),
        Err(e) => Err(format!("não deu para verificar atualizações: {e}")),
    }
}

pub async fn install(app: &AppHandle) -> Result<(), String> {
    if !auto_supported() {
        return Err("nesta instalação a atualização é manual".into());
    }
    let updater = app
        .updater_builder()
        .timeout(Duration::from_secs(120))
        .build()
        .map_err(|e| format!("atualizador: {e}"))?;
    let update = updater
        .check()
        .await
        .map_err(|e| format!("não deu para verificar: {e}"))?
        .ok_or_else(|| "já está na versão mais nova".to_string())?;
    log::info!("atualizando para {}", update.version);
    update
        .download_and_install(|_, _| {}, || {})
        .await
        .map_err(|e| format!("não deu para atualizar: {e}"))?;
    // No Windows o instalador já encerrou e reabriu o app; aqui é o AppImage.
    app.restart();
}
