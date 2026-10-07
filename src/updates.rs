use eframe::egui;
use sha2::{Digest, Sha256};
use std::{
    fs::File,
    io::Write,
    path::{Path, PathBuf},
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc, Mutex,
    },
    thread,
    time::Duration,
};

pub const RELEASES_URL: &str = "https://github.com/doomsdayradio/player/releases";
pub const BUILD_TAG: &str = match option_env!("DOOMSDAY_BUILD_TAG") {
    Some(tag) => tag,
    None => "local",
};
const DOWNLOAD_ROOT: &str = "https://github.com/doomsdayradio/player/releases/download/";
const MAX_DOWNLOAD: u64 = 64 * 1024 * 1024;
const LATEST_API: &str = "https://api.github.com/repos/doomsdayradio/player/releases/latest";
pub const CAN_INSTALL: bool = cfg!(all(
    any(target_os = "windows", target_os = "linux"),
    target_arch = "x86_64"
));

#[derive(Clone)]
pub enum State {
    Checking,
    Current,
    Available(Release),
    Downloading { received: u64, total: u64 },
    Installing,
    Restarting,
    Error(String),
}

impl State {
    pub fn busy(&self) -> bool {
        matches!(
            self,
            Self::Checking | Self::Downloading { .. } | Self::Installing | Self::Restarting
        )
    }
}

pub struct Updater {
    state: Arc<Mutex<State>>,
    cancelled: Arc<AtomicBool>,
    restart: Arc<AtomicBool>,
    context: egui::Context,
}

impl Updater {
    pub fn new(context: egui::Context, restart: Arc<AtomicBool>) -> Self {
        let updater = Self {
            state: Arc::new(Mutex::new(State::Current)),
            cancelled: Arc::new(AtomicBool::new(false)),
            restart,
            context,
        };
        updater.check();
        updater
    }

    pub fn state(&self) -> State {
        self.state.lock().unwrap().clone()
    }

    pub fn check(&self) {
        {
            let mut state = self.state.lock().unwrap();
            if state.busy() {
                return;
            }
            *state = State::Checking;
        }
        let state = self.state.clone();
        let cancelled = self.cancelled.clone();
        let context = self.context.clone();
        thread::spawn(move || {
            let result = run_async(async { latest_release(&client()?).await });
            if cancelled.load(Ordering::Relaxed) {
                return;
            }
            *state.lock().unwrap() = match result {
                Ok(release) if newer_release(&release.tag, BUILD_TAG) => State::Available(release),
                Ok(_) => State::Current,
                Err(error) => State::Error(error),
            };
            context.request_repaint();
        });
    }

    pub fn install(&self, release: Release) {
        if !CAN_INSTALL {
            return;
        }
        {
            let mut state = self.state.lock().unwrap();
            if state.busy() {
                return;
            }
            *state = State::Downloading {
                received: 0,
                total: release.size,
            };
        }
        let state = self.state.clone();
        let cancelled = self.cancelled.clone();
        let restart = self.restart.clone();
        let context = self.context.clone();
        thread::spawn(move || {
            let result = run_async(async {
                let directory = tempfile::tempdir().map_err(|_| {
                    "Temporärer Update-Ordner konnte nicht angelegt werden".to_string()
                })?;
                let download = directory.path().join(&release.asset);
                download_verified(&client()?, &release, &download, &cancelled, |received| {
                    *state.lock().unwrap() = State::Downloading {
                        received,
                        total: release.size,
                    };
                    context.request_repaint();
                })
                .await?;
                if cancelled.load(Ordering::Relaxed) {
                    return Err("Update abgebrochen".into());
                }
                *state.lock().unwrap() = State::Installing;
                context.request_repaint();
                let executable = prepare_executable(&download, directory.path())?;
                if cancelled.load(Ordering::Relaxed) {
                    return Err("Update abgebrochen".into());
                }
                self_replace::self_replace(executable).map_err(|_| "Programm konnte nicht ersetzt werden. Schreibrechte prüfen oder Release manuell herunterladen.".to_string())?;
                Ok(())
            });
            if cancelled.load(Ordering::Relaxed) {
                return;
            }
            match result {
                Ok(()) => {
                    *state.lock().unwrap() = State::Restarting;
                    restart.store(true, Ordering::SeqCst);
                    context.send_viewport_cmd(egui::ViewportCommand::Close);
                }
                Err(error) => *state.lock().unwrap() = State::Error(error),
            }
            context.request_repaint();
        });
    }
}

impl Drop for Updater {
    fn drop(&mut self) {
        self.cancelled.store(true, Ordering::Relaxed);
    }
}

fn run_async<T>(future: impl std::future::Future<Output = Result<T, String>>) -> Result<T, String> {
    tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .map_err(|_| "Update-Abfrage konnte nicht gestartet werden".to_string())?
        .block_on(future)
}

fn client() -> Result<reqwest::Client, String> {
    reqwest::Client::builder()
        .https_only(true)
        .connect_timeout(Duration::from_secs(8))
        .read_timeout(Duration::from_secs(20))
        .timeout(Duration::from_secs(180))
        .redirect(reqwest::redirect::Policy::limited(5))
        .user_agent(concat!("DoomsdayRadio/", env!("CARGO_PKG_VERSION")))
        .build()
        .map_err(|_| "Update-Abfrage konnte nicht gestartet werden".to_string())
}

async fn response(client: &reqwest::Client, url: &str) -> Result<reqwest::Response, String> {
    let response = client
        .get(url)
        .send()
        .await
        .map_err(|_| "Update-Server nicht erreichbar".to_string())?;
    if !response.status().is_success() {
        return Err(format!(
            "Update-Server: HTTP {}",
            response.status().as_u16()
        ));
    }
    Ok(response)
}

async fn bounded_bytes(
    client: &reqwest::Client,
    url: &str,
    limit: usize,
) -> Result<Vec<u8>, String> {
    let mut response = response(client, url).await?;
    let mut data = Vec::new();
    while let Some(chunk) = response
        .chunk()
        .await
        .map_err(|_| "Update-Download unterbrochen".to_string())?
    {
        if data.len().saturating_add(chunk.len()) > limit {
            return Err("Update-Antwort ist zu groß".into());
        }
        data.extend_from_slice(&chunk);
    }
    Ok(data)
}

async fn latest_release(client: &reqwest::Client) -> Result<Release, String> {
    let asset = platform_asset().ok_or("Kein Update für diese Plattform verfügbar")?;
    let data = bounded_bytes(client, LATEST_API, 1024 * 1024).await?;
    Release::parse(&data, asset)
}

async fn download_verified(
    client: &reqwest::Client,
    release: &Release,
    destination: &Path,
    cancelled: &AtomicBool,
    mut progress: impl FnMut(u64),
) -> Result<(), String> {
    let sums = bounded_bytes(client, &release.url("SHA256SUMS"), 16 * 1024).await?;
    let expected = expected_checksum(&sums, &release.asset)?;
    let mut response = response(client, &release.url(&release.asset)).await?;
    let mut file = File::create(destination)
        .map_err(|_| "Update-Datei konnte nicht angelegt werden".to_string())?;
    let mut hasher = Sha256::new();
    let mut received = 0u64;
    while let Some(chunk) = response
        .chunk()
        .await
        .map_err(|_| "Update-Download unterbrochen".to_string())?
    {
        if cancelled.load(Ordering::Relaxed) {
            return Err("Update abgebrochen".into());
        }
        received += chunk.len() as u64;
        if received > release.size || received > MAX_DOWNLOAD {
            return Err("Update-Download überschreitet die erwartete Größe".into());
        }
        file.write_all(&chunk)
            .map_err(|_| "Update-Datei konnte nicht geschrieben werden".to_string())?;
        hasher.update(&chunk);
        progress(received);
    }
    if received != release.size {
        return Err("Update-Download ist unvollständig".into());
    }
    verify_checksum(hasher, &expected)?;
    file.sync_all()
        .map_err(|_| "Update-Datei konnte nicht gespeichert werden".to_string())?;
    Ok(())
}

#[cfg(target_os = "windows")]
fn prepare_executable(download: &Path, _: &Path) -> Result<PathBuf, String> {
    Ok(download.to_owned())
}

#[cfg(target_os = "linux")]
fn prepare_executable(download: &Path, directory: &Path) -> Result<PathBuf, String> {
    use std::io::Read;
    use std::os::unix::fs::PermissionsExt;
    let file = File::open(download)
        .map_err(|_| "Update-Archiv konnte nicht geöffnet werden".to_string())?;
    let decoder = flate2::read::GzDecoder::new(file).take(MAX_DOWNLOAD * 2);
    let mut archive = tar::Archive::new(decoder);
    let target = directory.join("doomsday-radio");
    let mut found = false;
    for entry in archive
        .entries()
        .map_err(|_| "Ungültiges Update-Archiv".to_string())?
    {
        let mut entry = entry.map_err(|_| "Ungültiges Update-Archiv".to_string())?;
        if entry
            .path()
            .map_err(|_| "Ungültiger Archivpfad".to_string())?
            .as_ref()
            != Path::new("doomsday-radio-linux-x64")
        {
            continue;
        }
        if found || !entry.header().entry_type().is_file() || entry.size() > MAX_DOWNLOAD {
            return Err("Ungültiges Programm im Update-Archiv".into());
        }
        let mut output =
            File::create(&target).map_err(|_| "Update konnte nicht entpackt werden".to_string())?;
        std::io::copy(&mut entry, &mut output)
            .map_err(|_| "Update konnte nicht entpackt werden".to_string())?;
        output
            .sync_all()
            .map_err(|_| "Update konnte nicht gespeichert werden".to_string())?;
        std::fs::set_permissions(&target, std::fs::Permissions::from_mode(0o755))
            .map_err(|_| "Update-Dateirechte konnten nicht gesetzt werden".to_string())?;
        found = true;
    }
    if !found {
        return Err("Programm fehlt im Update-Archiv".into());
    }
    Ok(target)
}

#[cfg(not(any(target_os = "windows", target_os = "linux")))]
fn prepare_executable(_: &Path, _: &Path) -> Result<PathBuf, String> {
    Err("Auf dieser Plattform bitte das Release manuell ersetzen".into())
}

#[derive(Clone, Debug)]
pub struct Release {
    pub tag: String,
    pub asset: String,
    pub size: u64,
}

impl Release {
    fn parse(data: &[u8], asset: &str) -> Result<Self, String> {
        let release: serde_json::Value =
            serde_json::from_slice(data).map_err(|_| "Ungültige Release-Antwort".to_string())?;
        if release["draft"] != false || release["prerelease"] != false {
            return Err("Kein freigegebenes Release verfügbar".into());
        }
        let tag = release["tag_name"].as_str().ok_or("Release ohne Tag")?;
        if tag.is_empty()
            || !tag
                .bytes()
                .all(|byte| byte.is_ascii_alphanumeric() || b"-_.".contains(&byte))
            || tag == "."
            || tag == ".."
        {
            return Err("Ungültiger Release-Tag".into());
        }
        let assets = release["assets"]
            .as_array()
            .ok_or("Release ohne Downloads")?;
        if !assets.iter().any(|entry| entry["name"] == "SHA256SUMS") {
            return Err("Release enthält keine Prüfsummen".into());
        }
        let entry = assets
            .iter()
            .find(|entry| entry["name"] == asset)
            .ok_or("Kein Download für diese Plattform verfügbar")?;
        let size = entry["size"].as_u64().ok_or("Download ohne Größenangabe")?;
        if size == 0 || size > MAX_DOWNLOAD {
            return Err("Ungültige Download-Größe".into());
        }
        Ok(Self {
            tag: tag.to_owned(),
            asset: asset.to_owned(),
            size,
        })
    }

    pub fn url(&self, asset: &str) -> String {
        format!("{DOWNLOAD_ROOT}{}/{asset}", self.tag)
    }
}

fn platform_asset() -> Option<&'static str> {
    match (std::env::consts::OS, std::env::consts::ARCH) {
        ("windows", "x86_64") => Some("doomsday-radio-windows-x64.exe"),
        ("linux", "x86_64") => Some("doomsday-radio-linux-x64.tar.gz"),
        ("macos", "x86_64" | "aarch64") => Some("doomsday-radio-macos-universal.tar.gz"),
        _ => None,
    }
}

fn newer_release(candidate: &str, current: &str) -> bool {
    if candidate == current {
        return false;
    }
    let build = |tag: &str| -> Option<(u64, u64)> {
        let (number, attempt) = tag.strip_prefix("build-")?.split_once('-')?;
        Some((number.parse().ok()?, attempt.parse().ok()?))
    };
    if let (Some(candidate), Some(current)) = (build(candidate), build(current)) {
        return candidate > current;
    }
    let version = |tag: &str| semver::Version::parse(tag.strip_prefix('v').unwrap_or(tag));
    if let (Ok(candidate), Ok(current)) = (version(candidate), version(current)) {
        return candidate > current;
    }
    true
}

fn expected_checksum(data: &[u8], asset: &str) -> Result<String, String> {
    let text = std::str::from_utf8(data).map_err(|_| "Ungültige Prüfsummen-Datei".to_string())?;
    let mut found = None;
    for line in text.lines() {
        if let Some((digest, filename)) = line.split_once(char::is_whitespace) {
            if filename.trim().trim_start_matches('*') == asset {
                if found.is_some()
                    || digest.len() != 64
                    || !digest.bytes().all(|byte| byte.is_ascii_hexdigit())
                {
                    return Err("Ungültige oder doppelte Prüfsumme".into());
                }
                found = Some(digest.to_ascii_lowercase());
            }
        }
    }
    found.ok_or_else(|| "Prüfsumme für den Download fehlt".into())
}

fn verify_checksum(hasher: Sha256, expected: &str) -> Result<(), String> {
    let actual = format!("{:x}", hasher.finalize());
    if actual != expected {
        return Err("Prüfsumme stimmt nicht überein; Update wurde nicht installiert".into());
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn release_data(tag: &str, size: u64) -> Vec<u8> {
        serde_json::to_vec(&serde_json::json!({
            "tag_name": tag, "draft": false, "prerelease": false,
            "assets": [
                {"name": "doomsday-radio-windows-x64.exe", "size": size},
                {"name": "SHA256SUMS", "size": 300},
            ],
        }))
        .unwrap()
    }

    #[test]
    fn parses_the_existing_release_asset_names() {
        let release = Release::parse(
            &release_data("build-7-1", 7_000_000),
            "doomsday-radio-windows-x64.exe",
        )
        .unwrap();
        assert_eq!(release.tag, "build-7-1");
        assert_eq!(release.url(&release.asset), "https://github.com/doomsdayradio/player/releases/download/build-7-1/doomsday-radio-windows-x64.exe");
    }

    #[test]
    fn build_tags_and_versions_do_not_offer_downgrades() {
        assert!(newer_release("build-10-1", "build-9-2"));
        assert!(newer_release("build-10-2", "build-10-1"));
        assert!(!newer_release("build-9-2", "build-10-1"));
        assert!(!newer_release("build-10-1", "build-10-1"));
        assert!(newer_release("v0.1.10", "v0.1.9"));
        assert!(!newer_release("v0.1.9", "v0.1.10"));
        assert!(newer_release("build-7-1", "local"));
    }

    #[test]
    fn invalid_tags_sizes_and_missing_assets_are_rejected() {
        for tag in ["", "../other", "..", "tag?query"] {
            assert!(
                Release::parse(&release_data(tag, 100), "doomsday-radio-windows-x64.exe").is_err()
            );
        }
        for size in [0, MAX_DOWNLOAD + 1] {
            assert!(Release::parse(
                &release_data("build-7-1", size),
                "doomsday-radio-windows-x64.exe"
            )
            .is_err());
        }
        assert!(Release::parse(&release_data("build-7-1", 100), "missing.exe").is_err());
    }

    #[test]
    fn checksum_is_selected_by_exact_filename() {
        let digest = "ab".repeat(32);
        let data = format!("{digest}  other.exe\n{digest} *player.exe\n");
        assert_eq!(
            expected_checksum(data.as_bytes(), "player.exe").unwrap(),
            digest
        );
        assert!(expected_checksum(data.as_bytes(), "missing.exe").is_err());
        assert!(expected_checksum(
            format!("{digest}  player.exe\n{digest}  player.exe").as_bytes(),
            "player.exe"
        )
        .is_err());
    }

    #[test]
    fn checksum_mismatch_prevents_installation() {
        let mut hasher = Sha256::new();
        hasher.update(b"verified download");
        let expected = format!("{:x}", hasher.clone().finalize());
        assert!(verify_checksum(hasher.clone(), &expected).is_ok());
        assert!(verify_checksum(hasher, &"0".repeat(64)).is_err());
    }

    #[test]
    #[ignore = "Downloads the latest public release without installing it"]
    fn live_release_download_matches_its_checksum() {
        run_async(async {
            let client = client()?;
            let release = latest_release(&client).await?;
            let directory = tempfile::tempdir().map_err(|error| error.to_string())?;
            let destination = directory.path().join(&release.asset);
            download_verified(
                &client,
                &release,
                &destination,
                &AtomicBool::new(false),
                |_| {},
            )
            .await?;
            assert_eq!(std::fs::metadata(destination).unwrap().len(), release.size);
            Ok(())
        })
        .unwrap();
    }

    #[cfg(target_os = "linux")]
    #[test]
    fn linux_archive_extracts_only_the_expected_executable() {
        use std::io::Read;
        use std::os::unix::fs::PermissionsExt;
        let directory = tempfile::tempdir().unwrap();
        let download = directory.path().join("release.tar.gz");
        let encoder = flate2::write::GzEncoder::new(
            File::create(&download).unwrap(),
            flate2::Compression::default(),
        );
        let mut archive = tar::Builder::new(encoder);
        for (name, data) in [
            ("doomsday-radio.png", b"icon".as_slice()),
            ("doomsday-radio-linux-x64", b"executable".as_slice()),
        ] {
            let mut header = tar::Header::new_gnu();
            header.set_size(data.len() as u64);
            header.set_mode(0o755);
            header.set_cksum();
            archive.append_data(&mut header, name, data).unwrap();
        }
        archive.into_inner().unwrap().finish().unwrap();
        let executable = prepare_executable(&download, directory.path()).unwrap();
        let mut data = Vec::new();
        File::open(&executable)
            .unwrap()
            .read_to_end(&mut data)
            .unwrap();
        assert_eq!(data, b"executable");
        assert_eq!(
            std::fs::metadata(executable).unwrap().permissions().mode() & 0o777,
            0o755
        );
        assert!(!directory.path().join("doomsday-radio.png").exists());
    }

    #[cfg(target_os = "linux")]
    #[test]
    fn linux_archive_without_an_executable_is_rejected() {
        let directory = tempfile::tempdir().unwrap();
        let download = directory.path().join("release.tar.gz");
        let encoder = flate2::write::GzEncoder::new(
            File::create(&download).unwrap(),
            flate2::Compression::default(),
        );
        tar::Builder::new(encoder)
            .into_inner()
            .unwrap()
            .finish()
            .unwrap();
        assert!(prepare_executable(&download, directory.path()).is_err());
    }
}
