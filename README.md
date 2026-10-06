# Doomsday Radio

Kleiner nativer Rust-Player für den **[Doomsday-Livestream](https://stream.doomsday.radio/live)**.
Keine anderen Sender, kein Browser, keine Installation der App, keine
externen Assets, keine gespeicherten Einstellungen.

Start/Stopp, Lautstärke, Stummschaltung, Live-Status und automatische
Wiederverbindung. Der 48-Band-Visualizer zeigt das echte Audiosignal nach
Lautstärkeregelung, keine zufällige Animation. Leertaste startet/stoppt.
Das Fenster erscheint vor dem Verbindungsaufbau; Audio startet per Play.

## Portable Dateien

Der lokal erstellte Windows-Build liegt unter
[`dist/doomsday-radio-windows-x64.exe`](dist/doomsday-radio-windows-x64.exe):
6.8 MiB, nur Windows-System-DLLs laut Importprüfung des initialen Builds. Die Sichtprüfung und
der Mac-Build sind noch offen.

Der lokal erstellte Linux-Build liegt unter
[`dist/doomsday-radio-linux-x64`](dist/doomsday-radio-linux-x64): **9.1 MiB**,
ein einzelnes ELF-Binary für x86-64. Unter Debian 13 gebaut und getestet;
dieselbe Datei sowie derselbe Testsatz wurden unter Fedora 42 auf
Bibliothekskompatibilität geprüft. Auf beiden Systemen bestanden zehn
Unit-Tests und die Decodierung des echten Livestreams. Desktop-Fenster und
Audioausgabe unter Linux wurden nicht end-to-end geprüft.

Zwingende glibc-Baseline dieser lokalen Datei: **2.35**. Die Referenzen auf
glibc 2.39 sind als `WEAK` markiert und deshalb optional. Debian 12,
Ubuntu 22.04+, Linux Mint 21+ und aktuelle Fedora-/Arch-/openSUSE-Tumbleweed-
Installationen sind damit erwartete weitere Ziele, aber nicht alle hier
getestet. Die unten genannten Desktop-/Audiobibliotheken bleiben erforderlich.
Debian 11, Ubuntu 20.04, Alpine/musl und ARM-Systeme sind mit dieser Datei
nicht abgedeckt.

Eine einzelne Datei kann nicht unter allen drei Betriebssystemen laufen.
Der vorbereitete Workflow [.github/workflows/build.yml](.github/workflows/build.yml)
startet bei **Push**, **Pull Request** oder manuell über **Actions > Portable
builds > Run workflow**. Er erzeugt drei Downloads, die 30 Tage aufbewahrt
werden:

| Actions-Artefakt | Inhalt | Ausführbare Datei |
| --- | --- | --- |
| `doomsday-radio-windows-x64` | Windows-EXE | `doomsday-radio-windows-x64.exe` |
| `doomsday-radio-linux-x64` | `doomsday-radio-linux-x64.tar.gz` | `doomsday-radio-linux-x64` |
| `doomsday-radio-macos-universal` | `doomsday-radio-macos-universal.tar.gz` | `doomsday-radio-macos` |

Der Workflow nutzt Rust 1.99.0 und Cargo-Caches. Windows prüft Formatierung,
Unit-Tests und Clippy vor dem Build. Debian und macOS führen die Unit-Tests
aus; die netzwerk-/audioabhängigen Live-Tests bleiben ausgelassen. Der
Debian-12-Container erzeugt ein gemeinsames Linux-Binary; ein nachfolgender
Fedora-Job prüft daran Bibliotheks- und Symbolauflösung, nicht die grafische
Oberfläche oder Audioausgabe. macOS kombiniert Intel und Apple Silicon zu
einem Universal-Binary. Externe Actions sind auf Commit-Hashes festgelegt;
der Workflow benötigt nur Leserechte am Repository.

Den gewünschten Download im fertigen Actions-Lauf unter **Artifacts** laden
und zuerst den GitHub-ZIP-Download entpacken. Linux und macOS enthalten darin
ein `tar.gz`-Archiv, das die Ausführungsrechte erhält. Auch dieses entpacken:

```sh
tar -xzf doomsday-radio-linux-x64.tar.gz
./doomsday-radio-linux-x64
```

Auf macOS entsprechend `doomsday-radio-macos-universal.tar.gz` entpacken und
`./doomsday-radio-macos` starten. Die Archive sind nur Transport; die App
selbst bleibt jeweils eine einzelne Datei.

Die Action ist mit actionlint geprüft. Die Linux-Paketierung und der
Fedora-Ladercheck wurden lokal erfolgreich nachvollzogen. Ein vollständiger
GitHub-Actions-Lauf wurde hier noch nicht gestartet; der CI-Debian-12-Container
ist eine andere Buildbasis als die lokale Debian-13-WSL-Distribution.
macOS ist bis zur vom Nutzer später nachgereichten Repository-Adresse offen;
eine echte Mac-Datei ist deshalb noch nicht hergestellt worden.
Windows: EXE doppelklicken. Keine zusätzlichen DLL-Dateien oder VC-Runtime
nötig, die C-Laufzeit ist statisch eingebunden.

Lokaler Linux-Build: Falls das Download-Archiv die Ausführungsrechte entfernt hat:

```sh
chmod +x doomsday-radio-linux-x64
./doomsday-radio-linux-x64
```

Der CI-Linux-Build nutzt denselben Programmnamen wie der lokale Linux-Build;
auf macOS `doomsday-radio-macos` verwenden. Die Mac-Datei wird im Workflow
ad-hoc signiert, ist aber **nicht mit Developer ID signiert/notarisiert**.
Gatekeeper kann Internet-Downloads blockieren. Nur bei vertrauenswürdiger
Herkunft die Freigabe in Systemeinstellungen > Datenschutz & Sicherheit
bestätigen. Ohne Apple-Zertifikat ist ein garantiert warnungsfreier Erststart
nicht möglich. Die einzelne Mac-Datei ist kein Finder-App-Bundle; Start im
Terminal öffnet das native Fenster.

Debian benötigt eine grafische Sitzung, OpenGL und die normalen System-Audio-
und Fensterbibliotheken. Das ist eine portable Desktop-App, kein vollständig
statisches Linux-Binary für beliebige Minimalinstallationen. Falls nötig:

```sh
sudo apt install libasound2 libgl1 libegl1 libxkbcommon0 libx11-6 libxi6 libxrandr2 libwayland-client0 libwayland-cursor0 libwayland-egl1
```

Auf Fedora entsprechend:

```sh
sudo dnf install alsa-lib libX11 libXi libXrandr libxkbcommon libwayland-client libwayland-cursor libwayland-egl mesa-libGL mesa-libEGL
```

Weitere unkomplizierte Desktop-Ziele sind Ubuntu, Mint und aktuelle
Arch-/openSUSE-Tumbleweed-Systeme mit den genannten Bibliotheken. Ein
Linux-ARM64-Build, etwa für Raspberry Pi OS 64 Bit mit Desktop, wäre eine
nächste Erweiterung mit eigener Build-Datei; er ist hier nicht erzeugt worden.
Ohne grafische Sitzung wird eine separate TUI-Version benötigt; diese ist
noch nicht implementiert. Linux-Builds haben keinen eigenen Senderumfang:
auch hier ist ausschließlich Doomsday Radio möglich.

Windows- und Mac-Builds können ebenfalls Sicherheitswarnungen für nicht
zertifiziert signierte Downloads anzeigen. Internet und ein funktionierendes
Audioausgabegerät sind erforderlich. Ältere Debian-Versionen und Linux/Windows
auf ARM sind nicht Bestandteil dieser Build-Matrix.

## Lokal bauen

Rust >= 1.88, getestet mit 1.99.0. Windows benötigt beim **Bauen** MSVC Build
Tools; macOS die Xcode Command Line Tools. Auf Debian:

```sh
sudo apt install build-essential pkg-config libasound2-dev libx11-dev libxi-dev libxrandr-dev libxkbcommon-dev libwayland-dev libgl1-mesa-dev
```

```sh
cargo build --locked --release
cargo run --locked --release
```

Nur die ausführbare Datei aus `target/release/` ist zur Weitergabe nötig.
LTO, Größenoptimierung und entfernte Debugsymbole halten den Release klein.
Alle Schriften und Icons sind eingebettet. Keine Temp-Dateien oder Streamaufnahmen.
PCM-Puffer: maximal 12 MP3-Pakete; Decoder-Lesepuffer: 64 KiB. Das Fenster
zeichnet bei Wiedergabe mit maximal ca. 30 Hz neu und bleibt im Ruhezustand inaktiv.

## Prüfen

```sh
cargo fmt --check
cargo test --locked
cargo clippy --locked --all-targets -- -D warnings
```

Die beiden Live-Tests sind in CI bewusst ausgeschlossen: Sie brauchen Internet,
einer davon auch einen Audioausgang und spielt kurz mit 10 % Lautstärke.

```sh
cargo test --locked -- --ignored --nocapture
```

Verifiziert unter Windows: aktualisierter Release-Build, Formatierung, Clippy
und zehn Unit-Tests einschließlich eingebetteter Assets. Beide Live-Tests
einschließlich echter Audioausgabe und sauberem Shutdown bestanden vor der
reinen Stilanpassung; der Audiocode blieb dabei unverändert.
Ein automatischer Fenstertest wurde wegen inkompatibler .NET-10-Grafikreferenzen
auf Nutzerentscheidung nicht weiterverfolgt. Die Sichtprüfung des Fensters
bleibt offen. Der Linux-Release und dessen Tests wurden unter Debian 13/WSL
ausgeführt; Loader-/Symbolauflösung und derselbe Testsatz samt echter
Streamdecodierung bestanden unter Fedora 42/WSL. Der Test für echte
Linux-Audioausgabe bleibt ausgelassen. Ein vollständiger Desktop-/Audio-
End-to-end-Test auf diesen Zielsystemen steht noch aus. macOS und der
Debian-12-CI-Build sind weiterhin vorbereitet, nicht ausgeführt.
