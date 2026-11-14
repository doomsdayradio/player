# Doomsday Radio

## Die Doomsday-Welt

**Doomsday Radio ist eine Sendestation aus dem Jahr 2222** — Radio, Geschichten
und Signale aus einer Zukunft, die noch verhindert werden kann. Ein unabhängiges
Online-Radio als zusammenhängende Welt: automatisierte Sendungen, Musik,
Hörspiele, Spiele und Werkzeuge. Jede Komponente ist ein eigener Sender im
Netzwerk — dieser Service ist einer davon.

```text
SOURCE MATERIAL  --->  PRODUCTION  --->  TRANSMISSION  --->  YOU
      lore            services          radio/CDN         [::]
```

`NO SIGNAL IS EVER TRULY LOST // EST. 2222`

Ein kleiner Player für [Doomsday Radio](https://doomsday.radio).

![Oberfläche des Doomsday-Radio-Players](https://github.com/doomsdayradio/player/releases/latest/download/screenshot.png)

## Herunterladen

[Neueste Version für Windows, Linux und macOS](https://github.com/doomsdayradio/player/releases/latest).

- **Windows:** Die `.exe` herunterladen und öffnen.
- **macOS:** Das Archiv entpacken und `Doomsday Radio.app` öffnen (Intel und Apple Silicon).
- **Linux:** Das Archiv entpacken und `doomsday-radio-linux-x64` starten.

Unter Windows und Linux lassen sich Updates über das Update-Symbol im Player installieren.
Auf macOS die neue Version herunterladen und die bisherige App ersetzen.

## Selbst kompilieren

Benötigt werden [Rust](https://rustup.rs/) ab Version 1.88 und Git sowie
MSVC Build Tools (Windows), Xcode Command Line Tools (macOS) oder
Desktop- und Audio-Entwicklungspakete (Linux).

```sh
git clone https://github.com/doomsdayradio/player.git
cd player
cargo build --locked --release
```

Die fertige Programmdatei liegt unter `target/release`.
