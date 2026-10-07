# Release-Signierung

## Windows: SignPath Foundation vorbereiten

Die Integration ist vorbereitet, aber standardmäßig deaktiviert. Ohne
freigeschaltetes SignPath-Projekt werden weiterhin unsignierte EXEs veröffentlicht.
Eine gültige Signatur garantiert nicht, dass SmartScreen sofort auf jede Warnung
verzichtet: Auch die Reputation des Herausgebers und der Datei spielt eine Rolle.

### Voraussetzungen

1. Eine passende OSI-anerkannte Lizenz für den eigenen Programmcode festlegen.
   Aktuell fehlt eine Projektlizenz. Die Schriftlizenz ist keine Lizenz für die App.
   Rechte an eingebetteten Assets, insbesondere dem Senderlogo, separat klären.
2. Die [Bedingungen der SignPath Foundation](https://signpath.org/terms) prüfen
   und das Projekt unter <https://signpath.org/apply> anmelden. Die Aufnahme ist
   nicht garantiert und setzt unter anderem eine nachvollziehbare Reputation voraus.
3. Verantwortliche für Entwicklung, Reviews und Signierfreigaben benennen;
   Zwei-Faktor-Authentifizierung verwenden. Die geforderte Code signing policy
   mit Rollen, Datenschutzhinweis und Anbieterangabe nach Aufnahme veröffentlichen.
   Vorher nicht behaupten, das Projekt werde bereits von SignPath signiert.
4. In SignPath das GitHub-Repository und den GitHub-Build als vertrauenswürdige
   Quelle konfigurieren, die SignPath GitHub App einrichten und eine produktive
   Signierpolicy samt berechtigten Freigabeverantwortlichen anlegen.
5. Eine Artifact Configuration mit ZIP-Wurzel und genau der enthaltenen Datei
   `doomsday-radio-windows-x64.exe` einrichten. Der Workflow verwendet den
   standardmäßigen ZIP-Upload von `actions/upload-artifact`.
6. Die von SignPath geforderten Windows-Produktmetadaten und deren Prüfregeln
   abgleichen: Das Build-Skript bettet Produktname, Produktversion und Icon als
   Windows-Ressourcen ein. Weitere Anforderungen der Projektpolicy separat prüfen.
   Einen vertrauenswürdigen Zeitstempel in der Signierkonfiguration sicherstellen.

### GitHub-Konfiguration

Unter **Settings > Secrets and variables > Actions** hinterlegen:

| Typ | Name | Wert |
| --- | --- | --- |
| Secret | `SIGNPATH_API_TOKEN` | Token eines Submitters für dieses Projekt und diese Policy |
| Variable | `SIGNPATH_ORGANIZATION_ID` | Organisations-ID aus SignPath |
| Variable | `SIGNPATH_PROJECT_SLUG` | Projekt-Slug aus SignPath |
| Variable | `SIGNPATH_SIGNING_POLICY_SLUG` | Slug der produktiven Signierpolicy |
| Variable | `SIGNPATH_ARTIFACT_CONFIGURATION_SLUG` | Slug der ZIP-/EXE-Konfiguration |
| Variable | `SIGNPATH_ENABLED` | Erst nach vollständiger Einrichtung auf `true` setzen |

Tokens ausschließlich direkt als GitHub Secret eintragen, niemals in Code,
Issues oder Chat. Ein Testzertifikat ist kein Ersatz für die produktive Policy.

### Ablauf

Nur Hauptbranch- und `v…`-Tag-Builds werden zur Signierung eingereicht;
Pull Requests und Feature-Branches erhalten keine Signierfreigabe.
Der Workflow lädt die unsignierte EXE als separates kurzlebiges Artifact hoch,
reicht dessen ID bei SignPath ein und wartet bis zu 60 Minuten auf die
**manuelle Freigabe**, die das kostenlose Foundation-Angebot verlangt.
Bei Zeitüberschreitung den Lauf nach Klärung erneut starten.

Nach der Freigabe werden die zurückgelieferte EXE, die Authenticode-Signatur
und das Zeitstempelzertifikat geprüft. Erst danach wird das Windows-Artifact
für das Release hochgeladen. Fehlende Konfiguration, fehlgeschlagene Signierung
oder ungültige Signatur blockieren bei aktivierter Integration das Release;
es gibt keinen stillen Rückfall auf eine unsignierte EXE.
Prüfsummen werden anschließend aus den tatsächlich veröffentlichten Dateien erzeugt.
Die Release-Beschreibung kennzeichnet Windows als signiert oder unsigniert.

## macOS

Die aktuelle ad-hoc-Signatur schützt nicht vor Gatekeeper-Warnungen für Downloads.
Für öffentliche Distribution sind ein Apple-Developer-Program-Konto, ein
**Developer ID Application**-Zertifikat und Apples Notarisierung nötig.
Der typische Ablauf ist Signieren mit Hardened Runtime und Zeitstempel,
Notarisierung mit `notarytool` und Prüfung des Ergebnisses.

Der Workflow erzeugt bereits ein `.app`-Bundle mit Icon für den Start per
Doppelklick, verpackt als `tar.gz`. Ein DMG wäre eine zusätzliche Verpackungsoption.
Das Notarisierungsticket kann am unterstützten Bundle/DMG angeheftet werden;
am nackten Binary oder `tar.gz` lässt es sich nicht anheften.
macOS-Signierung und Notarisierung sind hier noch nicht eingerichtet.

## Linux

Für portable ELF-Dateien gibt es normalerweise keine zentrale Warnung wie
SmartScreen oder Gatekeeper und keine notwendige kommerzielle Codesignatur.
Ausführungsrechte, kompatible glibc und installierte Desktop-/Audiobibliotheken
bleiben erforderlich. Sicherheitsrichtlinien einzelner Systeme können strenger sein.

Die Releases enthalten bereits SHA-256-Prüfsummen zur Integritätskontrolle.
Prüfsummen allein bestätigen nicht den Herausgeber: Dafür wären zusätzlich
signierte Prüfsummen, beispielsweise mit GPG oder Sigstore, sinnvoll.
Bei späteren DEB-/RPM-Repositories sollten auch Pakete bzw. Repository-Metadaten
nach den Regeln des jeweiligen Paketmanagers signiert werden.
