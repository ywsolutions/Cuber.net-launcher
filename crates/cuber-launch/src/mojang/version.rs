//! JSON версии (<id>.json): что скачать и как запустить.
//!
//! Поддерживаются оба формата аргументов: старый `minecraftArguments` (до 1.13)
//! и новый `arguments.game/jvm` с правилами. Версии загрузчиков модов (Fabric и др.)
//! ссылаются на ванильную через `inheritsFrom` — см. [`VersionJson::merge_parent`].

use serde::{Deserialize, Serialize};

use super::library::Library;
use super::rules::{allowed, Env, Rule};
use super::FileRef;

#[derive(Debug, Clone, Default, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct VersionJson {
    pub id: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub inherits_from: Option<String>,
    #[serde(default, rename = "type", skip_serializing_if = "Option::is_none")]
    pub kind: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub main_class: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub minecraft_arguments: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub arguments: Option<Arguments>,
    #[serde(default)]
    pub libraries: Vec<Library>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub downloads: Option<Downloads>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub asset_index: Option<AssetIndexRef>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub assets: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub java_version: Option<JavaVersion>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub logging: Option<Logging>,
    /// Чей client.jar использовать (для унаследованных версий — родителя).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub jar: Option<String>,
}

#[derive(Debug, Clone, Default, Deserialize, Serialize)]
pub struct Arguments {
    #[serde(default)]
    pub game: Vec<Argument>,
    #[serde(default)]
    pub jvm: Vec<Argument>,
}

/// Аргумент: строка или строка/список строк с правилами.
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(untagged)]
pub enum Argument {
    Plain(String),
    Conditional { rules: Vec<Rule>, value: ArgValue },
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(untagged)]
pub enum ArgValue {
    One(String),
    Many(Vec<String>),
}

#[derive(Debug, Clone, Default, Deserialize, Serialize)]
pub struct Downloads {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub client: Option<FileRef>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AssetIndexRef {
    pub id: String,
    pub sha1: String,
    pub size: u64,
    #[serde(default)]
    pub total_size: Option<u64>,
    pub url: String,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct JavaVersion {
    /// Компонент рантайма Mojang: jre-legacy, java-runtime-alpha, -gamma, -delta…
    pub component: String,
    pub major_version: u32,
}

#[derive(Debug, Clone, Default, Deserialize, Serialize)]
pub struct Logging {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub client: Option<LoggingClient>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct LoggingClient {
    /// Например: -Dlog4j.configurationFile=${path}
    pub argument: String,
    pub file: FileRef,
}

impl Argument {
    /// Значения аргумента, если он применим в этом окружении.
    pub fn values(&self, env: &Env) -> Vec<&str> {
        match self {
            Argument::Plain(s) => vec![s.as_str()],
            Argument::Conditional { rules, value } if allowed(rules, env) => match value {
                ArgValue::One(s) => vec![s.as_str()],
                ArgValue::Many(v) => v.iter().map(String::as_str).collect(),
            },
            Argument::Conditional { .. } => vec![],
        }
    }
}

impl VersionJson {
    /// Наложить эту (дочернюю) версию на родительскую.
    /// Поля дочерней побеждают, библиотеки и аргументы объединяются;
    /// при совпадении библиотеки (группа:артефакт:классификатор) остаётся дочерняя.
    pub fn merge_parent(self, parent: VersionJson) -> VersionJson {
        let parent_jar = parent.jar_id().to_string();
        let own_client = self
            .downloads
            .as_ref()
            .and_then(|d| d.client.as_ref())
            .is_some();
        let mut libraries = self.libraries;
        let mut seen: std::collections::HashSet<String> =
            libraries.iter().map(|l| l.identity()).collect();
        for lib in parent.libraries {
            if seen.insert(lib.identity()) {
                libraries.push(lib);
            }
        }

        let arguments = match (parent.arguments, self.arguments) {
            (None, None) => None,
            (Some(a), None) | (None, Some(a)) => Some(a),
            (Some(mut p), Some(c)) => {
                p.game.extend(c.game);
                p.jvm.extend(c.jvm);
                Some(p)
            }
        };

        // client.jar: свой, если у дочерней версии есть собственный, иначе родительский.
        let jar = match self.jar {
            Some(j) => Some(j),
            None if own_client => Some(self.id.clone()),
            None => Some(parent_jar),
        };

        VersionJson {
            id: self.id,
            inherits_from: None,
            kind: self.kind.or(parent.kind),
            main_class: self.main_class.or(parent.main_class),
            minecraft_arguments: self.minecraft_arguments.or(parent.minecraft_arguments),
            arguments,
            libraries,
            downloads: self.downloads.or(parent.downloads),
            asset_index: self.asset_index.or(parent.asset_index),
            assets: self.assets.or(parent.assets),
            java_version: self.java_version.or(parent.java_version),
            logging: self.logging.or(parent.logging),
            jar,
        }
    }

    /// Id версии, чей client.jar запускается.
    pub fn jar_id(&self) -> &str {
        self.jar.as_deref().unwrap_or(&self.id)
    }

    /// Компонент Java: из JSON или jre-legacy (Java 8) для совсем старых версий.
    pub fn java_component(&self) -> (String, u32) {
        match &self.java_version {
            Some(j) => (j.component.clone(), j.major_version),
            None => ("jre-legacy".into(), 8),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_new_style_arguments() {
        let json = r#"{
            "id": "1.20.1",
            "mainClass": "net.minecraft.client.main.Main",
            "arguments": {
                "game": ["--username", "${auth_player_name}",
                         {"rules":[{"action":"allow","features":{"is_demo_user":true}}],"value":"--demo"}],
                "jvm": [{"rules":[{"action":"allow","os":{"name":"osx"}}],"value":["-XstartOnFirstThread"]},
                        "-cp", "${classpath}"]
            },
            "libraries": [],
            "javaVersion": {"component":"java-runtime-gamma","majorVersion":17}
        }"#;
        let v: VersionJson = serde_json::from_str(json).unwrap();
        let env = Env {
            os: "windows",
            arch: "x86_64",
            features: Default::default(),
        };
        let game: Vec<&str> = v
            .arguments
            .as_ref()
            .unwrap()
            .game
            .iter()
            .flat_map(|a| a.values(&env))
            .collect();
        assert_eq!(game, ["--username", "${auth_player_name}"]);
        let jvm: Vec<&str> = v
            .arguments
            .as_ref()
            .unwrap()
            .jvm
            .iter()
            .flat_map(|a| a.values(&env))
            .collect();
        assert_eq!(jvm, ["-cp", "${classpath}"]);
        assert_eq!(v.java_component(), ("java-runtime-gamma".into(), 17));
    }

    #[test]
    fn merge_child_over_parent() {
        let parent: VersionJson = serde_json::from_str(
            r#"{"id":"1.20.1","mainClass":"vanilla.Main",
                "libraries":[{"name":"org.ow2.asm:asm:9.3"},{"name":"com.mojang:brigadier:1.1.8"}],
                "arguments":{"game":["--a"],"jvm":["-Dx"]},
                "assets":"5"}"#,
        )
        .unwrap();
        let child: VersionJson = serde_json::from_str(
            r#"{"id":"fabric-loader-0.15-1.20.1","inheritsFrom":"1.20.1","mainClass":"fabric.Main",
                "libraries":[{"name":"org.ow2.asm:asm:9.6"},{"name":"net.fabricmc:fabric-loader:0.15.0"}],
                "arguments":{"game":[],"jvm":["-Dfabric"]}}"#,
        )
        .unwrap();
        let merged = child.merge_parent(parent);
        assert_eq!(merged.main_class.as_deref(), Some("fabric.Main"));
        assert_eq!(merged.jar_id(), "1.20.1");
        assert_eq!(merged.assets.as_deref(), Some("5"));
        let names: Vec<_> = merged.libraries.iter().map(|l| l.name.as_str()).collect();
        // asm из загрузчика вытесняет ванильный.
        assert_eq!(
            names,
            [
                "org.ow2.asm:asm:9.6",
                "net.fabricmc:fabric-loader:0.15.0",
                "com.mojang:brigadier:1.1.8"
            ]
        );
        assert_eq!(merged.arguments.unwrap().jvm.len(), 2);
    }
}
