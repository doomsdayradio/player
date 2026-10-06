# Lessons

- Bei ELF-Portabilität nicht nur die höchste GLIBC-Zahl ausgeben: die
  Versionsanforderungen mit readelf --version-info nach starken und WEAK-
  Symbolen unterscheiden. Der aktuelle Linux-Player braucht zwingend 2.35;
  seine 2.39-Referenzen sind optional. Andere Distributionen durch Loader-
  Prüfung und Ausführung desselben Testbinaries verifizieren.
- Schreibgeschützte WSL-Projektmounts nicht für Buildausgaben umkonfigurieren:
  nativen Buildcache verwenden und fertige Artefakte über Windows/WSL kopieren.
- Wenn Fabian eine Repository-Adresse später nachreichen möchte, keine
  weiteren Rückfragen zum Mac-Runner stellen. Unabhängige Linux-Builds
  abschließen und den Mac-Build bis zur Angabe des Repositories offen lassen.
- Fabian möchte den Player optisch an doomsday.radio orientiert haben.
  Zuerst die konkrete Vorlage betrachten; Farben, Logo und Typografie daraus
  ableiten statt eine unabhängige Neon-Palette zu verwenden. Eingebettete
  Assets beibehalten, damit ein portables Binary ohne Start-Downloads entsteht.
- Bei Symphonia MediaSourceStream den Standardpuffer verwenden: eigene Größen
  müssen Zweierpotenzen und strikt größer als 32 KiB sein. Mit einem echten
  nicht seekbaren Stream prüfen, nicht nur mit synthetischen Samples.
- .NET 10 verteilt System.Drawing-Typen auf weitere private Windows-Assemblies.
  Einen C#-Screenshot-Helfer nicht mit einer einzigen System.Drawing-Referenz
  als portabel deklarieren.
- Fabian entschied nach dem blockierten Fenstertest für Abschluss ohne weitere
  UI-Automation. Diese Entscheidung respektieren, defekte Testhilfen entfernen
  und offene Sichtprüfung transparent ausweisen.
