# Lessons

- Bei ELF-Portabilitaet nicht nur die hoechste GLIBC-Zahl ausgeben: die
  Versionsanforderungen mit readelf --version-info nach starken und WEAK-
  Symbolen unterscheiden. Der aktuelle Linux-Player braucht zwingend 2.35;
  seine 2.39-Referenzen sind optional. Andere Distributionen durch Loader-
  Pruefung und Ausfuehrung desselben Testbinaries verifizieren.
- Schreibgeschuetzte WSL-Projektmounts nicht fuer Buildausgaben umkonfigurieren:
  nativen Buildcache verwenden und fertige Artefakte ueber Windows/WSL kopieren.
- Wenn Fabian eine Repository-Adresse spaeter nachreichen moechte, keine
  weiteren Rueckfragen zum Mac-Runner stellen. Unabhaengige Linux-Builds
  abschliessen und den Mac-Build bis zur Angabe des Repositories offen lassen.
- Fabian moechte den Player optisch an doomsday.radio orientiert haben.
  Zuerst die konkrete Vorlage betrachten; Farben, Logo und Typografie daraus
  ableiten statt eine unabhaengige Neon-Palette zu verwenden. Eingebettete
  Assets beibehalten, damit ein portables Binary ohne Start-Downloads entsteht.
- Bei Symphonia MediaSourceStream den Standardpuffer verwenden: eigene Groessen
  muessen Zweierpotenzen und strikt groesser als 32 KiB sein. Mit einem echten
  nicht seekbaren Stream pruefen, nicht nur mit synthetischen Samples.
- .NET 10 verteilt System.Drawing-Typen auf weitere private Windows-Assemblies.
  Einen C#-Screenshot-Helfer nicht mit einer einzigen System.Drawing-Referenz
  als portabel deklarieren.
- Fabian entschied nach dem blockierten Fenstertest fuer Abschluss ohne weitere
  UI-Automation. Diese Entscheidung respektieren, defekte Testhilfen entfernen
  und offene Sichtpruefung transparent ausweisen.
