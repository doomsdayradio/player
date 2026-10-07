# Doomsday Radio

Ein kleiner nativer Player für [Doomsday Radio](https://doomsday.radio):
Start/Stopp, Lautstärke, Stummschaltung und Audio-Visualizer.
Ohne Browser oder Installation der App.

![Oberfläche des Doomsday-Radio-Players](https://github.com/doomsdayradio/player/releases/latest/download/screenshot.png)

## Downloads

[Neueste Downloads](https://github.com/doomsdayradio/player/releases/latest)
für Windows, Linux und macOS.
Jeder erfolgreiche Build auf dem Hauptbranch veröffentlicht automatisch ein
Release mit `build-…`-Tag; Versions-Tags wie `v0.1.0` werden ebenfalls veröffentlicht.

Windows und Linux: x64. macOS: Intel und Apple Silicon.
Archive entpacken. Unter Windows die EXE, unter macOS `Doomsday Radio.app` starten.
Unter Linux `./doomsday-radio-linux-x64` starten; optional mit
`bash install-desktop.sh` einen Menüeintrag mit Icon für den aktuellen Benutzer anlegen.
Nicht signierte Downloads können Sicherheitswarnungen auslösen.
Einrichtung und Status der [Release-Signierung](.github/SIGNING.md).

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
