//! Сквозная проверка: установка версии с (тестового) сервера и запуск «игры».
//!
//! Вместо Minecraft — крошечная Java-программа, которая печатает свои аргументы.
//! Так проверяется вся цепочка: манифест → JSON версии → библиотеки (Mojang и
//! Maven-стиль) → ассеты → classpath и аргументы → запуск процесса в папке профиля.
//! Тест пропускается, если в системе нет javac.

mod common;

use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::Command;

use common::{sha1_hex, Server};
use cuber_core::Paths;
use cuber_launch::mojang::rules::Env;
use cuber_launch::{launch, Endpoints, Installer, LaunchOptions, Player, Profile, ProfileStore};

const FAKE_MAIN: &str = r#"
public class FakeMain {
    public static void main(String[] a) throws Exception {
        System.out.println("FAKE GAME STARTED");
        System.out.println("ARGS " + String.join(" ", a));
        System.out.println("CWD " + new java.io.File(".").getCanonicalPath());
        System.out.println("CP " + System.getProperty("java.class.path"));
    }
}
"#;

fn which(cmd: &str) -> Option<PathBuf> {
    let out = Command::new(if cfg!(windows) { "where" } else { "which" })
        .arg(cmd)
        .output()
        .ok()?;
    let s = String::from_utf8(out.stdout).ok()?;
    s.lines().next().map(PathBuf::from).filter(|p| p.exists())
}

/// Скомпилировать FakeMain и упаковать в jar (zip без сжатия).
fn build_fake_client(dir: &Path) -> Vec<u8> {
    std::fs::write(dir.join("FakeMain.java"), FAKE_MAIN).unwrap();
    let status = Command::new("javac")
        .args(["--release", "8", "FakeMain.java"])
        .current_dir(dir)
        .status()
        .unwrap();
    assert!(status.success(), "javac не смог собрать FakeMain");
    let class = std::fs::read(dir.join("FakeMain.class")).unwrap();

    let mut buf = std::io::Cursor::new(Vec::new());
    {
        let mut zip = zip::ZipWriter::new(&mut buf);
        let opts = zip::write::SimpleFileOptions::default()
            .compression_method(zip::CompressionMethod::Stored);
        zip.start_file("FakeMain.class", opts).unwrap();
        zip.write_all(&class).unwrap();
        zip.finish().unwrap();
    }
    buf.into_inner()
}

#[tokio::test]
async fn install_and_launch_fake_version() {
    let Some(java) = which("java") else {
        eprintln!("нет java — тест пропущен");
        return;
    };
    if which("javac").is_none() {
        eprintln!("нет javac — тест пропущен");
        return;
    }

    let tmp = tempfile::tempdir().unwrap();
    let server = Server::start();
    let b = &server.base;

    // Файлы «версии».
    let client = build_fake_client(tmp.path());
    let lib = b"library one".to_vec();
    let maven_lib = b"fabric-style library".to_vec();
    let asset1 = br#"{"key":"value"}"#.to_vec();
    let asset2 = b"\x89PNG fake icon".to_vec();
    let (h1, h2) = (sha1_hex(&asset1), sha1_hex(&asset2));

    server.put("/client.jar", client.clone());
    server.put("/lib.jar", lib.clone());
    server.put(
        "/maven/net/example/fabricish/2.0/fabricish-2.0.jar",
        maven_lib.clone(),
    );
    server.put(&format!("/res/{}/{}", &h1[..2], h1), asset1.clone());
    server.put(&format!("/res/{}/{}", &h2[..2], h2), asset2.clone());

    let index = format!(
        r#"{{"objects":{{
            "minecraft/lang/en_us.json":{{"hash":"{h1}","size":{}}},
            "icons/icon.png":{{"hash":"{h2}","size":{}}}}}}}"#,
        asset1.len(),
        asset2.len()
    );
    server.put("/index.json", index.clone());

    let version = format!(
        r#"{{
        "id": "test-1",
        "type": "release",
        "mainClass": "FakeMain",
        "arguments": {{
            "game": ["--username", "${{auth_player_name}}", "--uuid", "${{auth_uuid}}",
                     "--gameDir", "${{game_directory}}", "--assetIndex", "${{assets_index_name}}",
                     {{"rules":[{{"action":"allow","features":{{"is_demo_user":true}}}}],"value":"--demo"}}],
            "jvm": ["-Djava.library.path=${{natives_directory}}", "-cp", "${{classpath}}"]
        }},
        "libraries": [
            {{"name":"com.example:lib:1.0","downloads":{{"artifact":{{
                "path":"com/example/lib/1.0/lib-1.0.jar","sha1":"{}","size":{},"url":"{b}/lib.jar"}}}}}},
            {{"name":"net.example:fabricish:2.0","url":"{b}/maven/"}},
            {{"name":"only.on:mars:1.0","rules":[{{"action":"allow","os":{{"name":"mars"}}}}]}}
        ],
        "downloads": {{"client": {{"sha1":"{}","size":{},"url":"{b}/client.jar"}}}},
        "assetIndex": {{"id":"t1","sha1":"{}","size":{},"url":"{b}/index.json"}},
        "assets": "t1",
        "javaVersion": {{"component":"java-runtime-gamma","majorVersion":17}}
    }}"#,
        sha1_hex(&lib),
        lib.len(),
        sha1_hex(&client),
        client.len(),
        sha1_hex(index.as_bytes()),
        index.len(),
    );
    server.put("/v/test-1.json", version.clone());
    let manifest = format!(
        r#"{{"latest":{{"release":"test-1","snapshot":"test-1"}},
            "versions":[{{"id":"test-1","type":"release","url":"{b}/v/test-1.json","sha1":"{}"}}]}}"#,
        sha1_hex(version.as_bytes())
    );
    server.put("/manifest.json", manifest);

    // Установщик, смотрящий на тестовый сервер.
    let paths = Paths::at(tmp.path().join("cuber"));
    paths.ensure().unwrap();
    let mut installer = Installer::new(paths.clone())
        .unwrap()
        .with_endpoints(Endpoints {
            version_manifest: format!("{b}/manifest.json"),
            java_runtimes: format!("{b}/no-java.json"),
            resources: format!("{b}/res"),
        });
    installer.java_override = Some(java.clone());

    let store = ProfileStore::new(paths.profiles());
    let profile = store.create(Profile::new("Тест", "test-1")).unwrap();
    let game_dir = store.game_dir(&profile.id);

    let prepared = installer
        .prepare("test-1", &game_dir, |_| {})
        .await
        .expect("установка");

    // Всё скачано и лежит в общих папках.
    assert_eq!(std::fs::read(&prepared.client_jar).unwrap(), client);
    assert!(paths
        .libraries()
        .join("com/example/lib/1.0/lib-1.0.jar")
        .exists());
    assert!(paths
        .libraries()
        .join("net/example/fabricish/2.0/fabricish-2.0.jar")
        .exists());
    assert!(paths
        .assets()
        .join("objects")
        .join(&h1[..2])
        .join(&h1)
        .exists());
    // Библиотека «только для Марса» не скачивалась и не попала в classpath.
    assert!(!prepared
        .classpath
        .iter()
        .any(|p| p.to_string_lossy().contains("mars")));
    assert_eq!(prepared.classpath.len(), 3);

    // Запуск.
    let opts = LaunchOptions {
        player: Player::offline("Tester"),
        game_dir: game_dir.clone(),
        memory_mb: 256,
        java_override: None,
        extra_jvm_args: vec![],
        resolution: None,
    };
    let cmd = launch::build(&prepared, &opts, &Env::current()).unwrap();
    let log = store.logs_dir(&profile.id).join("latest.log");
    let status = launch::spawn(&cmd, &log).unwrap().wait().unwrap();
    assert!(status.success());

    let out = std::fs::read_to_string(&log).unwrap();
    println!("{out}");
    assert!(out.contains("FAKE GAME STARTED"));
    let uuid = cuber_launch::offline::offline_uuid("Tester")
        .simple()
        .to_string();
    assert!(
        out.contains(&format!("--username Tester --uuid {uuid}")),
        "{out}"
    );
    assert!(out.contains("--assetIndex t1"));
    assert!(!out.contains("--demo"));
    assert!(
        !out.contains("${"),
        "остались неподставленные переменные: {out}"
    );
    // Игра запущена в папке игры профиля. Сравниваем хвост пути: на Windows
    // канонический путь бывает с префиксом \\?\ и в другом регистре.
    let cwd_line = out
        .lines()
        .find_map(|l| l.strip_prefix("CWD "))
        .expect("нет строки CWD")
        .replace('\\', "/");
    assert!(
        cwd_line.ends_with(&format!("profiles/{}/minecraft", profile.id)),
        "{cwd_line}"
    );

    // Второй профиль на той же версии: ничего не качается заново.
    let before = server.state.lock().unwrap().log.len();
    let other = store.create(Profile::new("Другой", "test-1")).unwrap();
    installer
        .prepare("test-1", &store.game_dir(&other.id), |_| {})
        .await
        .unwrap();
    let after = server.state.lock().unwrap().log.len();
    assert_eq!(before, after, "повторная установка не должна ничего качать");
}
