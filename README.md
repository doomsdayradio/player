# Doomsday Radio

Kleiner nativer Rust-Player fuer den **[Doomsday-Livestream](https://stream.doomsday.radio/live)**.
Keine anderen Sender, kein Browser, keine Installation der App, keine
externen Assets, keine gespeicherten Einstellungen.

Start/Stopp, Lautstaerke, Stummschaltung, Live-Status und automatische
Wiederverbindung. Der 48-Band-Visualizer zeigt das echte Audiosignal nach
Lautstaerkeregelung, keine zufaellige Animation. Leertaste startet/stoppt.
Das Fenster erscheint vor dem Verbindungsaufbau; Audio startet per Play.

## Gestaltung

An [doomsday.radio](https://doomsday.radio) orientiert: warmes Radioterminal,
Bernstein-/Rostfarben, dezente Scanlines und Skalenteilung, phosphorgruene
Spektrumbalken und das zentrierte Senderlogo. Das kompakte Fenster bleibt eine
native Rust-Oberflaeche, keine eingebettete Website.

Logo und Share Tech Mono sind direkt in der ausfuehrbaren Datei eingebettet;
beim Start werden weder Bilder noch Schriften heruntergeladen. Das Senderlogo
stammt von der [Senderseite](https://doomsday.radio/ddd_radio_logo.png) und wurde fuer das kleine
Fenster auf 384 x 391 Pixel reduziert. Share Tech Mono stammt aus Google Fonts
und steht unter der SIL Open Font License; der Lizenztext liegt in
[assets/ShareTechMono-OFL.txt](assets/ShareTechMono-OFL.txt) und ist auch ueber
das Kontextmenue des Logos in der App zugaenglich.

## Portable Dateien

Der lokal erstellte Windows-Build liegt unter
[`dist/doomsday-radio-windows-x64.exe`](dist/doomsday-radio-windows-x64.exe):
6.8 MiB, nur Windows-System-DLLs laut Importpruefung des initialen Builds. Die Sichtpruefung und
der Mac-Build sind noch offen.

Der lokal erstellte Linux-Build liegt unter
[`dist/doomsday-radio-linux-x64`](dist/doomsday-radio-linux-x64): **9.1 MiB**,
ein einzelnes ELF-Binary fuer x86-64. Unter Debian 13 gebaut und getestet;
dieselbe Datei sowie derselbe Testsatz wurden unter Fedora 42 auf
Bibliothekskompatibilitaet geprueft. Auf beiden Systemen bestanden zehn
Unit-Tests und die Decodierung des echten Livestreams. Desktop-Fenster und
Audioausgabe unter Linux wurden nicht end-to-end geprueft.

Zwingende glibc-Baseline dieser lokalen Datei: **2.35**. Die Referenzen auf
glibc 2.39 sind als `WEAK` markiert und deshalb optional. Debian 12,
Ubuntu 22.04+, Linux Mint 21+ und aktuelle Fedora-/Arch-/openSUSE-Tumbleweed-
Installationen sind damit erwartete weitere Ziele, aber nicht alle hier
getestet. Die unten genannten Desktop-/Audiobibliotheken bleiben erforderlich.
Debian 11, Ubuntu 20.04, Alpine/musl und ARM-Systeme sind mit dieser Datei
nicht abgedeckt.

Eine einzelne Datei kann nicht unter allen drei Betriebssystemen laufen.
Der vorbereitete Workflow [.github/workflows/build.yml](.github/workflows/build.yml)
startet bei **Push**, **Pull Request** oder manuell ueber **Actions > Portable
builds > Run workflow**. Er erzeugt drei Downloads, die 30 Tage aufbewahrt
werden:

| Actions-Artefakt | Inhalt | Ausfuehrbare Datei |
| --- | --- | --- |
| `doomsday-radio-windows-x64` | Windows-EXE | `doomsday-radio-windows-x64.exe` |
| `doomsday-radio-linux-x64` | `doomsday-radio-linux-x64.tar.gz` | `doomsday-radio-linux-x64` |
| `doomsday-radio-macos-universal` | `doomsday-radio-macos-universal.tar.gz` | `doomsday-radio-macos` |

Der Workflow nutzt Rust 1.99.0 und Cargo-Caches. Windows prueft Formatierung,
Unit-Tests und Clippy vor dem Build. Debian und macOS fuehren die Unit-Tests
aus; die netzwerk-/audioabhaengigen Live-Tests bleiben ausgelassen. Der
Debian-12-Container erzeugt ein gemeinsames Linux-Binary; ein nachfolgender
Fedora-Job prueft daran Bibliotheks- und Symbolaufloesung, nicht die grafische
Oberflaeche oder Audioausgabe. macOS kombiniert Intel und Apple Silicon zu
einem Universal-Binary. Externe Actions sind auf Commit-Hashes festgelegt;
der Workflow benoetigt nur Leserechte am Repository.

Den gewuenschten Download im fertigen Actions-Lauf unter **Artifacts** laden
und zuerst den GitHub-ZIP-Download entpacken. Linux und macOS enthalten darin
ein `tar.gz`-Archiv, das die Ausfuehrungsrechte erhaelt. Auch dieses entpacken:

```sh
tar -xzf doomsday-radio-linux-x64.tar.gz
./doomsday-radio-linux-x64
```

Auf macOS entsprechend `doomsday-radio-macos-universal.tar.gz` entpacken und
`./doomsday-radio-macos` starten. Die Archive sind nur Transport; die App
selbst bleibt jeweils eine einzelne Datei.

Die Action ist mit actionlint geprueft. Die Linux-Paketierung und der
Fedora-Ladercheck wurden lokal erfolgreich nachvollzogen. Ein vollstaendiger
GitHub-Actions-Lauf wurde hier noch nicht gestartet; der CI-Debian-12-Container
ist eine andere Buildbasis als die lokale Debian-13-WSL-Distribution.
macOS ist bis zur vom Nutzer spaeter nachgereichten Repository-Adresse offen;
eine echte Mac-Datei ist deshalb noch nicht hergestellt worden.
Windows: EXE doppelklicken. Keine zusaetzlichen DLL-Dateien oder VC-Runtime
noetig, die C-Laufzeit ist statisch eingebunden.

Lokaler Linux-Build: Falls das Download-Archiv die Ausfuehrungsrechte entfernt hat:

```sh
chmod +x doomsday-radio-linux-x64
./doomsday-radio-linux-x64
```

Der CI-Linux-Build nutzt denselben Programmnamen wie der lokale Linux-Build;
auf macOS `doomsday-radio-macos` verwenden. Die Mac-Datei wird im Workflow
ad-hoc signiert, ist aber **nicht mit Developer ID signiert/notarisiert**.
Gatekeeper kann Internet-Downloads blockieren. Nur bei vertrauenswuerdiger
Herkunft die Freigabe in Systemeinstellungen > Datenschutz & Sicherheit
bestaetigen. Ohne Apple-Zertifikat ist ein garantiert warnungsfreier Erststart
nicht moeglich. Die einzelne Mac-Datei ist kein Finder-App-Bundle; Start im
Terminal oeffnet das native Fenster.

Debian benoetigt eine grafische Sitzung, OpenGL und die normalen System-Audio-
und Fensterbibliotheken. Das ist eine portable Desktop-App, kein vollstaendig
statisches Linux-Binary fuer beliebige Minimalinstallationen. Falls noetig:

```sh
sudo apt install libasound2 libgl1 libegl1 libxkbcommon0 libx11-6 libxi6 libxrandr2 libwayland-client0 libwayland-cursor0 libwayland-egl1
```

Auf Fedora entsprechend:

```sh
sudo dnf install alsa-lib libX11 libXi libXrandr libxkbcommon libwayland-client libwayland-cursor libwayland-egl mesa-libGL mesa-libEGL
```

Weitere unkomplizierte Desktop-Ziele sind Ubuntu, Mint und aktuelle
Arch-/openSUSE-Tumbleweed-Systeme mit den genannten Bibliotheken. Ein
Linux-ARM64-Build, etwa fuer Raspberry Pi OS 64 Bit mit Desktop, waere eine
naechste Erweiterung mit eigener Build-Datei; er ist hier nicht erzeugt worden.
Ohne grafische Sitzung wird eine separate TUI-Version benoetigt; diese ist
noch nicht implementiert. Linux-Builds haben keinen eigenen Senderumfang:
auch hier ist ausschliesslich Doomsday Radio moeglich.

Windows- und Mac-Builds koennen ebenfalls Sicherheitswarnungen fuer nicht
zertifiziert signierte Downloads anzeigen. Internet und ein funktionierendes
Audioausgabegeraet sind erforderlich. Aeltere Debian-Versionen und Linux/Windows
auf ARM sind nicht Bestandteil dieser Build-Matrix.

## Lokal bauen

Rust >= 1.88, getestet mit 1.99.0. Windows benoetigt beim **Bauen** MSVC Build
Tools; macOS die Xcode Command Line Tools. Auf Debian:

```sh
sudo apt install build-essential pkg-config libasound2-dev libx11-dev libxi-dev libxrandr-dev libxkbcommon-dev libwayland-dev libgl1-mesa-dev
```

```sh
cargo build --locked --release
cargo run --locked --release
```

Nur die ausfuehrbare Datei aus `target/release/` ist zur Weitergabe noetig.
LTO, Groessenoptimierung und entfernte Debugsymbole halten den Release klein.
Alle Schriften und Icons sind eingebettet. Keine Temp-Dateien oder Streamaufnahmen.
PCM-Puffer: maximal 12 MP3-Pakete; Decoder-Lesepuffer: 64 KiB. Das Fenster
zeichnet bei Wiedergabe mit maximal ca. 30 Hz neu und bleibt im Ruhezustand inaktiv.

## Pruefen

```sh
cargo fmt --check
cargo test --locked
cargo clippy --locked --all-targets -- -D warnings
```

Die beiden Live-Tests sind in CI bewusst ausgeschlossen: Sie brauchen Internet,
einer davon auch einen Audioausgang und spielt kurz mit 10 % Lautstaerke.

```sh
cargo test --locked -- --ignored --nocapture
```

Verifiziert unter Windows: aktualisierter Release-Build, Formatierung, Clippy
und zehn Unit-Tests einschliesslich eingebetteter Assets. Beide Live-Tests
einschliesslich echter Audioausgabe und sauberem Shutdown bestanden vor der
reinen Stilanpassung; der Audiocode blieb dabei unveraendert.
Ein automatischer Fenstertest wurde wegen inkompatibler .NET-10-Grafikreferenzen
auf Nutzerentscheidung nicht weiterverfolgt. Die Sichtpruefung des Fensters
bleibt offen. Der Linux-Release und dessen Tests wurden unter Debian 13/WSL
ausgefuehrt; Loader-/Symbolaufloesung und derselbe Testsatz samt echter
Streamdecodierung bestanden unter Fedora 42/WSL. Der Test fuer echte
Linux-Audioausgabe bleibt ausgelassen. Ein vollstaendiger Desktop-/Audio-
End-to-end-Test auf diesen Zielsystemen steht noch aus. macOS und der
Debian-12-CI-Build sind weiterhin vorbereitet, nicht ausgefuehrt.
