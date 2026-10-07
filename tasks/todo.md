# Doomsday Radio Player

## Vereinbarter Umfang

Native Rust-App, ausschließlich <https://stream.doomsday.radio/live>.
Kompaktes Fenster, Start/Stopp, Lautstärke, Stummschaltung, echter
FFT-Visualizer, automatische Wiederverbindung. Keine Browser-Laufzeit,
keine Konfiguration, keine Aufnahmen und keine weiteren Sender.
Je Betriebssystem/Architektur eine ausführbare Datei. Windows lokal
verifizieren; macOS und Debian über native CI-Runner bauen. Linux setzt
die normalen Desktop-/Audiobibliotheken voraus; macOS-Builds sind unsigniert.

## Plan

- [x] Build-Action mit Cache, portablen Archiven und Fedora-Check aktualisieren.
- [x] Workflow mit actionlint und passenden lokalen Paketierungschecks validieren.
- [x] Downloadnamen und Auslöser der Build-Action dokumentieren.
- [x] Stil von doomsday.radio mit eingebettetem Logo und Originalschrift umsetzen.
- [x] Eingebettete Assets, Compiler, Clippy und aktualisierte Windows-EXE prüfen.
- [x] Player, begrenztes Streaming und FFT implementieren.
- [x] Unit-Tests, Live-Stream und echte Windows-Audioausgabe prüfen.
- [x] Optimierten Windows-Build erzeugen.
- [x] Build-Pipeline für Windows, Debian und beide Mac-Architekturen erstellen.
- [x] Portabilitätsgrenzen und Startanleitung dokumentieren.
- [ ] Manuelle Sichtprüfung des Windows-Fensters (offen).
- [x] Linux-Release unter Debian 13/WSL bauen und zehn Unit-Tests ausführen.
- [x] Bibliotheken, Symbole und gleichen Testsatz unter Fedora 42/WSL prüfen.
- [x] Echte Streamdecodierung unter Debian und Fedora prüfen.
- [x] Linux-Datei und Plattform-Kompatibilität dokumentieren.
- [ ] Mac-Build nach nachgereichter GitHub-Repository-Adresse ausführen (offen).
- [ ] Debian-12-CI-Build und vollständige Linux-Desktop-/Audio-Prüfung (offen).

## Signierte Release-Prüfsummen

- [ ] Signierverfahren festlegen: GPG oder alternativ Sigstore mit GitHub-OIDC.
- [ ] Bei GPG einen dedizierten Release-Signierschlüssel erstellen, Ablaufdatum festlegen und Backup sowie Widerruf vorbereiten.
- [ ] Öffentlichen GPG-Schlüssel veröffentlichen und den Fingerprint über einen unabhängigen, vertrauenswürdigen Kanal bekanntgeben.
- [ ] Privaten GPG-Signierschlüssel und gegebenenfalls die Passphrase direkt als geschützte GitHub Secrets hinterlegen; niemals ins Repository oder in den Chat übernehmen.
- [ ] Signierung nur für vertrauenswürdige Release-Jobs freigeben; Pull Requests und Feature-Branches ausschließen.
- [ ] Die endgültige `SHA256SUMS` nach sämtlichen Binary-Signierungen mit GPG signieren und `SHA256SUMS.asc` zusammen mit den Downloads veröffentlichen.
- [ ] Bei aktivierter Prüfsummensignierung das Release bei fehlenden Secrets oder Signierfehlern blockieren; temporären Schlüsselbund nach Gebrauch entfernen.
- [ ] Verifikation in `.github/SIGNING.md` dokumentieren: Fingerprint prüfen, Schlüssel importieren, Signatur prüfen und anschließend Datei-Prüfsummen vergleichen.
- [ ] Positivtest mit echtem Release sowie Negativtests mit veränderten Prüfsummen, manipuliertem Download und falschem Schlüssel durchführen.

## Prüfhypothese

Der bestätigte MP3-Livestream lässt sich ohne seekbare Datei mit Symphonia
decodieren. Ein begrenzter PCM-Kanal liefert an Rodio; ausschließlich
verbrauchte Samples speisen die FFT. Unit-Tests prüfen Frequenzzuordnung,
Pufferunterlauf und Abbruch, ein opt-in Live-Test die echte Decodierung.

## Ergebnis

Build-Action aktualisiert: Push, Pull Request und manuelle Auslösung.
Drei Artefakte (Windows x64, Linux x64, Mac universal), 30 Tage Aufbewahrung,
Cargo-Caches, feste Action-Commit-Hashes und maximale Joblaufzeiten.
Unix-Tararchive erhalten Ausführungsrechte. Nachfolgender Fedora-Job prüft
das Debian-Binary mit ldd -r. actionlint 1.7.12 meldet keine Fehler;
Linux-Paketierung/Archivmodus und Fedora-Ladercheck lokal erfolgreich.
Vollständiger GitHub-Lauf bleibt bis zur Repository-Anbindung ausstehend.

Zehn Unit-Tests nach Stilanpassung bestanden; zwei opt-in Live-Tests zuvor bestanden. MP3 vom echten Sender
decodiert, PCM auf dem Windows-Audioausgang verbraucht, Netzwerk-Worker nach
Stoppen beendet. cargo fmt --check und Clippy mit -D warnings bestanden.
Optimierte Windows-EXE erfolgreich gebaut, statische CRT aktiviert.
Aktualisierte Ausgabe: dist/doomsday-radio-windows-x64.exe, 7,164,416 Bytes (6.8 MiB).
PE-Importprüfung des initialen Builds zeigt ausschließlich Windows-System-DLLs, keine VC-Runtime.

Website-Vorlage unter doomsday.radio betrachtet. Warme Bernstein-/Rostpalette,
zentriertes Original-Logo, Share Tech Mono, Scanlines, Skala und segmentiertes
phosphorgrünes Spektrum umgesetzt. Logo auf 384 x 391 Pixel optimiert;
Logo, Schrift und Schriftlizenz im Binary eingebettet. Kein Assetdownload beim
Start. Audiocode unverändert, Clippy -D warnings und Release-Build grün.

Der Screenshot-Helfer scheiterte an .NET-10-Assembly-Referenzen. Nutzer entschied
ausdrücklich für Abschluss ohne automatischen Fenstertest; der defekte Helfer
wurde entfernt. UI-Sichtprüfung daher nicht als bestanden behaupten.
Die GitHub-Actions-Pipeline ist vorbereitet, nicht ausgeführt. Mac ist
ad-hoc-signiert, nicht notarisiert; Debian setzt Desktop-Systembibliotheken
voraus. Windows- und Linux-Build sind lokal hergestellt. Der Linux-Release
liegt als dist/doomsday-radio-linux-x64 vor: 9,506,360 Bytes (9.1 MiB), ELF64
x86-64. Buildbasis Debian 13/WSL; Bibliotheks-/Symbolauflösung, zehn Unit-Tests
und echte HTTPS-/MP3-Decodierung auch unter Fedora 42/WSL erfolgreich.
Zwingende glibc-Baseline 2.35; neuere 2.39-Symbole sind WEAK/optional.
Andere genannte Distributionen sind erwartete, nicht vollständig getestete
Ziele. GUI und Audioausgabe unter Linux bleiben end-to-end ungeprüft.

Debian-Buildwerkzeuge und Laufzeitbibliotheken in WSL installiert; Rust 1.99.0
unter /home/fab1/.cargo/bin. Buildcache /home/fab1/.cache/doomsday-radio-target
hält Windows-Builds getrennt. Der Windows-Projektmount in Debian ist
schreibgeschützt; Artefakt daher über den Windows-Zugriff auf WSL kopiert.
Lokale Buildtask in .vscode/tasks.json angelegt. Der Nutzer reicht die
GitHub-Repository-Adresse für macOS später nach; bis dahin keine weiteren
Rückfragen und kein vorgetäuschter Mac-Build.
