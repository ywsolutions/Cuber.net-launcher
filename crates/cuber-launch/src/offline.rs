//! Офлайн-игрок: UUID вычисляется из ника так же, как это делает сам Minecraft
//! (`UUID.nameUUIDFromBytes("OfflinePlayer:" + ник)`), поэтому миры и инвентари
//! узнают игрока при любом лаунчере.

use md5::{Digest, Md5};
use uuid::Uuid;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Player {
    pub name: String,
    pub uuid: Uuid,
    pub access_token: String,
    /// legacy (офлайн), msa (Microsoft).
    pub user_type: String,
}

impl Player {
    pub fn offline(name: &str) -> Self {
        Self {
            name: name.to_string(),
            uuid: offline_uuid(name),
            // Игре нужен непустой токен; для офлайн-режима значение не проверяется.
            access_token: "0".into(),
            user_type: "legacy".into(),
        }
    }
}

pub fn offline_uuid(name: &str) -> Uuid {
    let mut bytes: [u8; 16] = Md5::digest(format!("OfflinePlayer:{name}").as_bytes()).into();
    bytes[6] = (bytes[6] & 0x0f) | 0x30; // версия 3
    bytes[8] = (bytes[8] & 0x3f) | 0x80; // вариант IETF
    Uuid::from_bytes(bytes)
}

/// Проверка ника: 3–16 символов, латиница, цифры, подчёркивание.
pub fn is_valid_nickname(name: &str) -> bool {
    (3..=16).contains(&name.len()) && name.chars().all(|c| c.is_ascii_alphanumeric() || c == '_')
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn matches_java_name_uuid_from_bytes() {
        // Значение, которое выдаёт Java для "OfflinePlayer:Notch".
        assert_eq!(
            offline_uuid("Notch").to_string(),
            "b50ad385-829d-3141-a216-7e7d7539ba7f"
        );
    }

    #[test]
    fn nickname_rules() {
        assert!(is_valid_nickname("SYGEN"));
        assert!(is_valid_nickname("a_b_1"));
        assert!(!is_valid_nickname("ab"));
        assert!(!is_valid_nickname("имя"));
        assert!(!is_valid_nickname("with space"));
        assert!(!is_valid_nickname("seventeen_chars__"));
    }
}
