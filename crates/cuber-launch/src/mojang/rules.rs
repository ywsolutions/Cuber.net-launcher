//! Правила из JSON версии: какие библиотеки и аргументы нужны на этой системе.

use std::collections::HashMap;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum Action {
    Allow,
    Disallow,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
pub struct Rule {
    pub action: Action,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub os: Option<OsRule>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub features: Option<HashMap<String, bool>>,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
pub struct OsRule {
    #[serde(default)]
    pub name: Option<String>,
    #[serde(default)]
    pub arch: Option<String>,
    #[serde(default)]
    pub version: Option<String>,
}

/// Окружение, для которого проверяются правила.
#[derive(Debug, Clone)]
pub struct Env {
    /// windows, linux, osx — как в JSON Mojang.
    pub os: &'static str,
    /// x86 (32 бита), x86_64, arm64.
    pub arch: &'static str,
    pub features: HashMap<String, bool>,
}

impl Env {
    /// Текущая система, все необязательные возможности выключены.
    pub fn current() -> Self {
        let os = match std::env::consts::OS {
            "windows" => "windows",
            "macos" => "osx",
            _ => "linux",
        };
        let arch = match std::env::consts::ARCH {
            "x86" => "x86",
            "aarch64" => "arm64",
            _ => "x86_64",
        };
        Self {
            os,
            arch,
            features: HashMap::new(),
        }
    }

    pub fn is_64bit(&self) -> bool {
        self.arch != "x86"
    }
}

/// Разрешено ли по набору правил. Пустой набор — разрешено.
/// Правила применяются по порядку, побеждает последнее подходящее.
pub fn allowed(rules: &[Rule], env: &Env) -> bool {
    if rules.is_empty() {
        return true;
    }
    let mut result = false;
    for rule in rules {
        if matches(rule, env) {
            result = rule.action == Action::Allow;
        }
    }
    result
}

fn matches(rule: &Rule, env: &Env) -> bool {
    if let Some(os) = &rule.os {
        if let Some(name) = &os.name {
            if name != env.os {
                return false;
            }
        }
        if let Some(arch) = &os.arch {
            // В JSON Mojang "x86" означает 32-битную систему.
            let ok = match arch.as_str() {
                "x86" => env.arch == "x86",
                other => other == env.arch,
            };
            if !ok {
                return false;
            }
        }
        // os.version (регулярка по версии ОС) встречается только в старых
        // правилах для macOS 10.5 — считаем, что подходит.
    }
    if let Some(features) = &rule.features {
        for (name, wanted) in features {
            if env.features.get(name).copied().unwrap_or(false) != *wanted {
                return false;
            }
        }
    }
    true
}

#[cfg(test)]
mod tests {
    use super::*;

    fn env(os: &'static str) -> Env {
        Env {
            os,
            arch: "x86_64",
            features: HashMap::new(),
        }
    }

    fn rules(json: &str) -> Vec<Rule> {
        serde_json::from_str(json).unwrap()
    }

    #[test]
    fn empty_rules_allow() {
        assert!(allowed(&[], &env("windows")));
    }

    #[test]
    fn allow_all_except_osx() {
        let r = rules(r#"[{"action":"allow"},{"action":"disallow","os":{"name":"osx"}}]"#);
        assert!(allowed(&r, &env("windows")));
        assert!(allowed(&r, &env("linux")));
        assert!(!allowed(&r, &env("osx")));
    }

    #[test]
    fn only_on_windows() {
        let r = rules(r#"[{"action":"allow","os":{"name":"windows"}}]"#);
        assert!(allowed(&r, &env("windows")));
        assert!(!allowed(&r, &env("linux")));
    }

    #[test]
    fn x86_means_32bit() {
        let r = rules(r#"[{"action":"allow","os":{"arch":"x86"}}]"#);
        assert!(!allowed(&r, &env("windows")));
        let mut e = env("windows");
        e.arch = "x86";
        assert!(allowed(&r, &e));
    }

    #[test]
    fn features_must_match() {
        let r = rules(r#"[{"action":"allow","features":{"is_demo_user":true}}]"#);
        assert!(!allowed(&r, &env("linux")));
        let mut e = env("linux");
        e.features.insert("is_demo_user".into(), true);
        assert!(allowed(&r, &e));
    }
}
