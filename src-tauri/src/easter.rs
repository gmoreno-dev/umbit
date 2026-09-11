//! Um segredo pequeno. Se você chegou aqui lendo o código, parabéns:
//! é assim mesmo que se acha. As assinaturas dentro do Macintosh de 1984
//! também estavam à vista de quem abrisse a carcaça.

use chrono::{Datelike, Local};
use unicode_normalization::UnicodeNormalization;

use crate::{config::EasterEggConfig, models::Track};

/// O nome, normalizado: sem acento, sem espaço, minúsculo.
const HER: &str = "analivia";

/// A música favorita dela.
pub const HERS: (&str, &str) = ("Matilda", "Harry Styles");
/// A nossa.
pub const OURS: (&str, &str) = ("Sign of the Times", "Harry Styles");

pub const NOTE: &str = "primeiro a sua, depois a nossa";
pub const ARTIST_TEASE: &str = "Harry Styles";

#[derive(Debug, Clone)]
pub struct EggTracks {
    pub hers: Track,
    pub ours: Track,
}

/// Remove acentos, espaços e pontuação; deixa só letras e dígitos minúsculos.
pub fn normalize(q: &str) -> String {
    q.nfd()
        .filter(|c| c.is_ascii_alphanumeric())
        .map(|c| c.to_ascii_lowercase())
        .collect()
}

pub fn matches(query: &str) -> bool {
    normalize(query) == HER
}

pub fn is_birthday() -> bool {
    let today = Local::now().date_naive();
    today.month() == 3 && today.day() == 18
}

/// Contas para as quais o bilhete existe, guardadas como SHA-256 do nome de
/// usuário em minúsculas, para o repositório não expor os nomes. Quem quiser
/// o próprio segredo troca estes hashes ou usa `users` na configuração.
const BUILTIN_USER_HASHES: &[&str] = &[];

fn sha256_hex(s: &str) -> String {
    use sha2::{Digest, Sha256};
    let mut h = Sha256::new();
    h.update(s.as_bytes());
    h.finalize().iter().map(|b| format!("{b:02x}")).collect()
}

pub fn enabled(cfg: &EasterEggConfig, username: Option<&str>) -> bool {
    match cfg.mode.as_str() {
        "always" => true,
        "never" => false,
        _ => match username {
            Some(u) => {
                let u = u.trim().to_lowercase();
                cfg.users.iter().any(|x| x.trim().to_lowercase() == u)
                    || BUILTIN_USER_HASHES.contains(&sha256_hex(&u).as_str())
            }
            None => false,
        },
    }
}

/// A linha que aparece na busca: a música dela, marcada para virar o bilhete.
pub fn decorate(hers: &Track) -> Track {
    let mut t = hers.clone();
    t.artists = vec![ARTIST_TEASE.to_string()];
    t.extra = Some("egg".to_string());
    t
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn normaliza_nome() {
        assert!(matches("Ana Lívia"));
        assert!(matches("ana livia"));
        assert!(matches("ANALIVIA"));
        assert!(matches("  ana  lívia "));
        assert!(!matches("ana"));
        assert!(!matches("livia"));
    }

    #[test]
    fn gatilho_por_conta() {
        let mut cfg = EasterEggConfig::default();
        assert!(!enabled(&cfg, Some("alguem")));
        cfg.users = vec!["Moreno".into()];
        assert!(enabled(&cfg, Some("moreno")));
        assert!(!enabled(&cfg, None));
        cfg.mode = "always".into();
        assert!(enabled(&cfg, None));
        cfg.mode = "never".into();
        assert!(!enabled(&cfg, Some("moreno")));
    }

    #[test]
    fn hash_estavel() {
        assert_eq!(sha256_hex("abc"), "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad");
    }
}
