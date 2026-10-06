# Doomsday Radio Player

## Vereinbarter Umfang

Native Rust-App, ausschliesslich <https://stream.doomsday.radio/live>.
Kompaktes Fenster, Start/Stopp, Lautstaerke, Stummschaltung, echter
FFT-Visualizer, automatische Wiederverbindung. Keine Browser-Laufzeit,
keine Konfiguration, keine Aufnahmen und keine weiteren Sender.
Je Betriebssystem/Architektur eine ausfuehrbare Datei. Windows lokal
verifizieren; macOS und Debian ueber native CI-Runner bauen. Linux setzt
die normalen Desktop-/Audiobibliotheken voraus; macOS-Builds sind unsigniert.

## Plan

- [x] Build-Action mit Cache, portablen Archiven und Fedora-Check aktualisieren.
- [x] Workflow mit actionlint und passenden lokalen Paketierungschecks validieren.
- [x] Downloadnamen und Ausloeser der Build-Action dokumentieren.
- [x] Stil von doomsday.radio mit eingebettetem Logo und Originalschrift umsetzen.
- [x] Eingebettete Assets, Compiler, Clippy und aktualisierte Windows-EXE pruefen.
- [x] Player, begrenztes Streaming und FFT implementieren.
- [x] Unit-Tests, Live-Stream und echte Windows-Audioausgabe pruefen.
- [x] Optimierten Windows-Build erzeugen.
- [x] Build-Pipeline fuer Windows, Debian und beide Mac-Architekturen erstellen.
- [x] Portabilitaetsgrenzen und Startanleitung dokumentieren.
- [ ] Manuelle Sichtpruefung des Windows-Fensters (offen).
- [x] Linux-Release unter Debian 13/WSL bauen und zehn Unit-Tests ausfuehren.
- [x] Bibliotheken, Symbole und gleichen Testsatz unter Fedora 42/WSL pruefen.
- [x] Echte Streamdecodierung unter Debian und Fedora pruefen.
- [x] Linux-Datei und Plattform-Kompatibilitaet dokumentieren.
- [ ] Mac-Build nach nachgereichter GitHub-Repository-Adresse ausfuehren (offen).
- [ ] Debian-12-CI-Build und vollstaendige Linux-Desktop-/Audio-Pruefung (offen).

## Pruefhypothese

Der bestaetigte MP3-Livestream laesst sich ohne seekbare Datei mit Symphonia
decodieren. Ein begrenzter PCM-Kanal liefert an Rodio; ausschliesslich
verbrauchte Samples speisen die FFT. Unit-Tests pruefen Frequenzzuordnung,
Pufferunterlauf und Abbruch, ein opt-in Live-Test die echte Decodierung.

## Ergebnis

Build-Action aktualisiert: Push, Pull Request und manuelle Ausloesung.
Drei Artefakte (Windows x64, Linux x64, Mac universal), 30 Tage Aufbewahrung,
Cargo-Caches, feste Action-Commit-Hashes und maximale Joblaufzeiten.
Unix-Tararchive erhalten Ausfuehrungsrechte. Nachfolgender Fedora-Job prueft
das Debian-Binary mit ldd -r. actionlint 1.7.12 meldet keine Fehler;
Linux-Paketierung/Archivmodus und Fedora-Ladercheck lokal erfolgreich.
Vollstaendiger GitHub-Lauf bleibt bis zur Repository-Anbindung ausstehend.

Zehn Unit-Tests nach Stilanpassung bestanden; zwei opt-in Live-Tests zuvor bestanden. MP3 vom echten Sender
decodiert, PCM auf dem Windows-Audioausgang verbraucht, Netzwerk-Worker nach
Stoppen beendet. cargo fmt --check und Clippy mit -D warnings bestanden.
Optimierte Windows-EXE erfolgreich gebaut, statische CRT aktiviert.
Aktualisierte Ausgabe: dist/doomsday-radio-windows-x64.exe, 7,164,416 Bytes (6.8 MiB).
PE-Importpruefung des initialen Builds zeigt ausschliesslich Windows-System-DLLs, keine VC-Runtime.

Website-Vorlage unter doomsday.radio betrachtet. Warme Bernstein-/Rostpalette,
zentriertes Original-Logo, Share Tech Mono, Scanlines, Skala und segmentiertes
phosphorgruenes Spektrum umgesetzt. Logo auf 384 x 391 Pixel optimiert;
Logo, Schrift und Schriftlizenz im Binary eingebettet. Kein Assetdownload beim
Start. Audiocode unveraendert, Clippy -D warnings und Release-Build gruen.

Der Screenshot-Helfer scheiterte an .NET-10-Assembly-Referenzen. Nutzer entschied
ausdruecklich fuer Abschluss ohne automatischen Fenstertest; der defekte Helfer
wurde entfernt. UI-Sichtpruefung daher nicht als bestanden behaupten.
Die GitHub-Actions-Pipeline ist vorbereitet, nicht ausgefuehrt. Mac ist
ad-hoc-signiert, nicht notarisiert; Debian setzt Desktop-Systembibliotheken
voraus. Windows- und Linux-Build sind lokal hergestellt. Der Linux-Release
liegt als dist/doomsday-radio-linux-x64 vor: 9,506,360 Bytes (9.1 MiB), ELF64
x86-64. Buildbasis Debian 13/WSL; Bibliotheks-/Symbolaufloesung, zehn Unit-Tests
und echte HTTPS-/MP3-Decodierung auch unter Fedora 42/WSL erfolgreich.
Zwingende glibc-Baseline 2.35; neuere 2.39-Symbole sind WEAK/optional.
Andere genannte Distributionen sind erwartete, nicht vollstaendig getestete
Ziele. GUI und Audioausgabe unter Linux bleiben end-to-end ungeprueft.

Debian-Buildwerkzeuge und Laufzeitbibliotheken in WSL installiert; Rust 1.99.0
unter /home/fab1/.cargo/bin. Buildcache /home/fab1/.cache/doomsday-radio-target
haelt Windows-Builds getrennt. Der Windows-Projektmount in Debian ist
schreibgeschuetzt; Artefakt daher ueber den Windows-Zugriff auf WSL kopiert.
Lokale Buildtask in .vscode/tasks.json angelegt. Der Nutzer reicht die
GitHub-Repository-Adresse fuer macOS spaeter nach; bis dahin keine weiteren
Rueckfragen und kein vorgetaeuschter Mac-Build.
