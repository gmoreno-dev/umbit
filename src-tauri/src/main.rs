// Sem console no Windows em release.
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

fn main() {
    // No Linux, o renderizador DMA-BUF do WebKitGTK custa uns 25 MB a mais de
    // memória sem ganho visível numa interface de duas cores. Quem quiser pode
    // sobrescrever exportando a variável antes de abrir o app.
    #[cfg(target_os = "linux")]
    if std::env::var_os("WEBKIT_DISABLE_DMABUF_RENDERER").is_none() {
        std::env::set_var("WEBKIT_DISABLE_DMABUF_RENDERER", "1");
    }
    umbit_lib::run()
}
