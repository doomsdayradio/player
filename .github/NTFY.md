# ntfy in Release-Builds

GitHub-Variablen: `NTFY_URL` ist die HTTPS-Serveradresse, `NTFY_TOPIC` das Topic.
Das GitHub-Secret `NTFY_TOKEN` wird als Lesetoken in Release-Programme eingebaut.
Die Downloads zeigen damit Titel ohne lokale Einrichtung an.

## Sicherheitsgrenze

Ein eingebauter Token ist aus dem Programm auslesbar und daher nicht mehr geheim.
Die zugehörige Identität darf ausschließlich das vorgesehene Topic lesen.
Keine Schreibrechte, weiteren Topics oder Adminrechte vergeben.

Der Workflow gibt den Token nur an Release-Build-Schritte auf dem Hauptbranch
oder bei `v…`-Tags weiter, nicht an Pull Requests oder andere Branches.
Er wird nicht in den Quellcode oder separate Konfigurationsdateien geschrieben.
GitHub-Caches enthalten nur Abhängigkeiten, keine kompilierten Programme oder
Compiler-Metadaten. Neue Cache-Schlüssel vermeiden alte Caches mit `target`.
Die veröffentlichten Programme enthalten den Token weiterhin absichtlich.

Zur Rotation das GitHub-Secret ersetzen und ein neues Release bauen.
Den alten Token serverseitig widerrufen; alte Downloads verlieren dann den Titelzugriff.
Lokale `NTFY_TOKEN`- oder `DOOMSDAY_NTFY_TOKEN`-Werte haben weiterhin Vorrang.
