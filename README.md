# Doomsday Radio

Ein kleiner nativer Player für [Doomsday Radio](https://doomsday.radio):
Start/Stopp, Lautstärke, Stummschaltung und Audio-Visualizer.
Ohne Browser oder Installation der App.

## Downloads

[GitHub Releases](https://github.com/doomsdayradio/player/releases)
für veröffentlichte Versionen.
Bis dahin: [Actions-Builds](https://github.com/doomsdayradio/player/actions/workflows/build.yml),
einen erfolgreichen Lauf öffnen und unter **Artifacts** herunterladen.

Windows und Linux: x64. macOS: Intel und Apple Silicon.
Archive entpacken und die ausführbare Datei starten.
Nicht signierte Downloads können Sicherheitswarnungen auslösen.

## Selbst kompilieren

[Rust](https://rustup.rs/) ab Version 1.88 installieren.
Windows benötigt MSVC Build Tools, macOS die Xcode Command Line Tools.
Unter Debian/Ubuntu zusätzlich:

```sh
sudo apt install build-essential pkg-config libasound2-dev libx11-dev libxi-dev libxrandr-dev libxkbcommon-dev libwayland-dev libgl1-mesa-dev
```

```sh
git clone https://github.com/doomsdayradio/player.git
cd player
cargo build --locked --release
```

Die fertige Datei liegt unter `target/release/doomsday-radio`
(Windows: `doomsday-radio.exe`).
