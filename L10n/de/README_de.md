[English](../../README.md) | [简体中文](../zh-cn/README_zh-CN.md) | [Español](../es/README_es.md) | [Português (BR)](../pt-BR/README_pt-BR.md) | [日本語](../ja/README_ja.md) | [Русский](../ru/README_ru.md) | [Français](../fr/README_fr.md) | **Deutsch** | [한국어](../ko/README_ko.md) | [Türkçe](../tr/README.tr.md) | [Bahasa Indonesia](../id/README.id.md)

# FamTree

Erstellt und gepflegt von **[john3sobieski](https://github.com/john3sobieski)**. Repository: <https://github.com/john3sobieski/family-tree>

Eine Desktop-App für Windows zum Erstellen, Bearbeiten und Durchsuchen deines Stammbaums sowie eine Android-App, um ihn auf dem Smartphone anzusehen. Alles liegt in einer einzigen `.famtree`-Datei: Personen, Beziehungen und alle Fotos.

> **Status:** einsatzbereit. Das Projekt gilt als abgeschlossen; künftig sind nur noch kleine Korrekturen zu erwarten. Lade den Windows-Installer oder die Android-APK unter [Releases](https://github.com/john3sobieski/family-tree/releases) herunter (siehe [Installation](#installation)).

## Funktionen

- **Drei Modi**
  - **Ansicht** – schreibgeschützte Mindmap mit flüssigem Zoomen und Verschieben.
  - **Bearbeiten** – Personen, Fotos und Beziehungen hinzufügen und ändern.
  - **Liste** – eine sortier- und filterbare Tabelle aller Personen im Stammbaum.
- **Mindmap, die um dich herum aufgebaut wird.** Markiere eine Person als „ich“, und der Baum wird von deinen obersten Vorfahren abwärts gezeichnet. Eltern, Geschwister, Onkel, Tanten, Cousins und Cousinen sowie entferntere Verwandte erscheinen dadurch automatisch. Beide Seiten der Familie werden nebeneinander angeordnet, auch die Familien der angeheirateten Verwandten werden einbezogen.
- **Fotokarten.** Jede Karte zeigt ein Passfoto, den Vornamen, den Nachnamen (bei Frauen mit dem Mädchennamen in Klammern), den Vatersnamen, einen Farbstreifen für das Geschlecht und bis zu zwei Flaggen für die Staatsangehörigkeit.
- **Personenfenster.** Ein Klick auf eine Karte öffnet ein größeres Fenster mit dem vollständigen Foto, einer Galerie (10+ Fotos pro Person sind kein Problem), Daten, Alter, Geburts- und Sterbeort, Beruf, Notizen und anklickbaren Links zu Eltern, Partnern und Kindern.
- **Suche.** Gib einen Namen ein, und die Kamera fliegt zu dieser Person auf der Karte und hebt sie hervor. Im Listenmodus funktioniert die Suche als Filter. `Strg+F` setzt den Fokus auf das Suchfeld.
- **Beziehungen.** Eltern, Partner, Kinder und Geschwister lassen sich mit einem Klick hinzufügen oder mit bereits vorhandenen Personen verknüpfen. Zyklen und doppelte Verknüpfungen werden abgelehnt.
- **Unvollständige Daten.** Du kennst nur das Jahr? Kein Problem, jeder Teil eines Datums kann leer bleiben.
- **Staatsangehörigkeiten.** Wähle pro Person bis zu zwei aus einer eingebauten Liste von 197 Ländern (alle UN-Mitglieder plus Vatikan, Palästina, Taiwan und Kosovo). Die Flaggen zeichnet die App selbst.
- **Für große Bäume gebaut.** Es werden nur sichtbare Karten gezeichnet, Vorschaubilder werden per Lazy Loading nachgeladen, und das Layout wird nur neu berechnet, wenn sich die Struktur ändert. Getestet mit Bäumen aus Tausenden von Personen.
- **Autosave.** Sobald der Baum eine Datei hat, wird jede Änderung kurz nach dem Aufhören mit dem Tippen automatisch gespeichert. Für einen neuen Baum nutze zuerst **Speichern unter…**. Vor dem Schließen wirst du außerdem gefragt, wenn etwas nicht gespeichert werden konnte.
- **Oberflächensprachen.** Русский, English, 中文, Español, Français, Deutsch und Polski. Über das Menü **Sprache** wechselst du die Sprache; die Auswahl wird gespeichert. Beim ersten Start folgt die App der Windows-Anzeigesprache und verwendet sonst Englisch. Auch die Ländernamen werden übersetzt.
- **Export.** **Datei → Exportieren…** speichert den ganzen Baum als PNG oder als Vektor-PDF, mit Rand an den Seiten. Bei PNG wählen Sie den Hintergrund (transparent, weiß oder schwarz) sowie dunkle oder helle Farben; die Größe wird automatisch so gewählt, dass alles scharf bleibt (bis ×2). Das PDF lässt sich ohne Qualitätsverlust zoomen. Sehr große Bäume exportiert man besser als PDF.
- **Doppelklick zum Öffnen.** Im Menü **Datei** kannst du `.famtree`-Dateien mit der App verknüpfen.
- **Dunkles und helles Design.** Umschalten mit der Sonne/Mond-Schaltfläche rechts in der Statusleiste; die Auswahl wird gespeichert.
- **Android-App (nur Ansicht).** Öffne dieselbe `.famtree`-Datei auf Smartphone oder Tablet. Die Karte sieht genau so aus wie am PC (gleiches Layout, gleiche Karten, Flaggen und Linien) und bietet Verschieben und Pinch-Zoom, Suche mit Kameraflug zur Person, eine Personenseite im Vollbild mit Fotos und Links zu Verwandten, **Einpassen** sowie ein helles/dunkles Design. Die Datei wird nie verändert: Bearbeiten, Listenmodus und Export bleiben dem PC vorbehalten. Die Oberfläche folgt der Sprache des Geräts (dieselben sieben Sprachen, sonst Englisch). Siehe [Installation → Android](#android).

## Installation

### Windows

Die App läuft unter Windows 10 und Windows 11. Es ist derselbe Installer und dieselbe exe für beide.

1. Öffne die Seite [Releases](https://github.com/john3sobieski/family-tree/releases) und lade **`FamTree-Setup-<Version>.exe`** herunter.
2. Starte die Datei. Sie installiert nur für deinen Benutzer, Administratorrechte sind nicht nötig. Im Assistenten kannst du wählen, dass `.famtree`-Dateien mit FamTree geöffnet werden, und eine Desktopverknüpfung anlegen.
3. Starte **FamTree** über das Startmenü.

Hinweise:

- Windows zeigt möglicherweise eine SmartScreen-Warnung an, weil Installer und `.exe` nicht signiert sind. Wähle *Weitere Informationen → Trotzdem ausführen*.
- Zum Aktualisieren startest du ein neueres Setup über die alte Version. Deine Stammbäume und Einstellungen bleiben erhalten.
- Zum Deinstallieren nutze *Einstellungen → Apps*. Deine `.famtree`-Dateien werden nie gelöscht.

### Android

Die Android-App zeigt Bäume nur an und verändert deine Datei nie. Sie braucht Android 10 oder neuer auf einem 64-Bit-ARM-Gerät (praktisch alle aktuellen Smartphones und Tablets).

1. Lade **`FamTree-<Version>.apk`** von der Seite [Releases](https://github.com/john3sobieski/family-tree/releases) direkt auf dein Gerät herunter.
2. Öffne die Datei. Android fragt einmalig, ob Apps aus dieser Quelle (Browser oder Dateimanager) installiert werden dürfen; bestätige das.
3. Bringe deine `.famtree`-Datei aufs Gerät (Cloud-Speicher, Messenger, USB-Kabel). Öffne sie dann im Dateimanager mit **Öffnen mit → FamTree**, oder starte FamTree und tippe auf **Öffnen**, um die Datei auszuwählen. Die zuletzt geöffnete Datei wird gemerkt. Öffnest du eine andere `.famtree`, während die App läuft, wird sie ohne Neustart übernommen.

Hinweise:

- Die App gibt es nur als APK, nicht über Google Play. Play Protect warnt eventuell vor einer App eines unbekannten Entwicklers; wähle trotzdem Installieren.
- Damit sie sich erneut öffnen lässt, behält die App eine private Kopie der zuletzt geöffneten Datei; das Original wird nie verändert.

### Selbst bauen

Den Installer bekommst du auch von GitHub Actions, ohne Werkzeuge zu installieren (ein GitHub-Konto ist nötig): Erstelle einen Fork des [Repositorys](https://github.com/john3sobieski/family-tree), öffne **Actions → Build → Run workflow** und lade das Artefakt **`famtree-setup`** aus dem fertigen Lauf herunter (standardmäßig 90 Tage gespeichert).

Die Android-APK entsteht auf dieselbe Weise: **Actions → Android → Run workflow**, danach das Artefakt **`famtree-android-<Version>`** herunterladen.

## Schnellstart

1. Starte `famtree.exe`. Mit **Datei → Neu** beginnst du deinen eigenen Baum, mit **Datei → Öffnen** öffnest du einen vorhandenen.
2. Wechsle in den Modus **Bearbeiten** und füge die erste Person hinzu (**＋ Neue Person**). In einem leeren Baum wird sie automatisch zu „ich“.
3. Wähle eine Person aus und nutze **＋ Elternteil**, **＋ Partner**, **＋ Kind** oder **＋ Geschwister**, um den Baum wachsen zu lassen. Namen, Daten, Fotos und Notizen trägst du rechts ein.
4. Wähle **Speichern unter…** und vergib einen Namen. Ab dann werden Änderungen automatisch gespeichert.
5. Wechsle zu **Ansicht**, um die Karte zu erkunden: Ziehen zum Verschieben, Scrollen zum Zoomen, **Einpassen**, um alles zu sehen, und ein Klick auf eine Karte für Details.

**Unter Android:** Öffne eine `.famtree`-Datei (siehe [Installation → Android](#android)), ziehe zum Verschieben und zoome mit zwei Fingern. **Einpassen** zeigt alles auf einmal, die Lupe öffnet die Personensuche, und ein Tipp auf eine Karte öffnet die Personenseite (ein Tipp auf einen Verwandten springt zu ihm). Die System-Zurück-Taste bzw. -Geste schließt Seite oder Suche Schritt für Schritt; auf der Karte schickt sie die App in den Hintergrund. Die Sonne/Mond-Schaltfläche wechselt das Design (standardmäßig folgt die App dem System).

## Die `.famtree`-Datei

Eine `.famtree`-Datei ist ein gewöhnliches ZIP-Archiv:

```
tree.json            people and relationships (plain, human-readable text)
media/<hash>.jpg     photos, resized to at most 1600 px
thumb/<hash>.jpg     small thumbnails for cards
```

- Die Fotos sind in der Datei gespeichert, du kannst also eine einzige `.famtree` an Verwandte schicken, ohne dass etwas verloren geht. Identische Fotos werden nur einmal gespeichert.
- `tree.json` ist mit jedem Texteditor lesbar und enthält Kommentare, die jede Beziehung beschreiben. Wenn du sie von Hand bearbeiten willst, entpacke das Archiv, bearbeite die Datei und packe sie wieder zusammen. Direktes Bearbeiten innerhalb der `.famtree` zerstört das Archiv.
- Ältere Dateien lassen sich weiterhin in neueren Versionen der App öffnen.

## Bekannte Einschränkungen

- Die Übersetzungen der Oberfläche (außer Russisch) wurden noch nicht von Muttersprachlern geprüft. Korrekturen sind über Issues willkommen.
- Datumsangaben werden unabhängig von der Sprache immer als `TT.MM.JJJJ` angezeigt.
- Noch kein Rückgängig.
- Der Import vieler Fotos auf einmal kann das Fenster kurz einfrieren lassen.
- HEIC-Fotos werden nicht unterstützt (JPEG/JFIF, PNG, WebP und BMP schon).
- Das Vorschaubild auf einer Karte ist ein zentrierter Ausschnitt, und den Ausschnitt kann man noch nicht selbst wählen.
- In sehr großen Familien kann das Layout Personen immer noch an merkwürdigen Stellen platzieren oder Linien zwischen Zweigen kreuzen (siehe Roadmap).
- Staatsangehörigkeiten werden im Listenmodus nicht angezeigt.
- Android: nur Ansicht (kein Bearbeiten, kein Listenmodus, kein Export), 64-Bit-ARM-Geräte mit Android 10 oder neuer, nur als APK (nicht bei Google Play).
- Android: Fotos lassen sich noch nicht per Pinch-Geste vergrößern, und es gibt kein Sprachmenü: Die App folgt der Systemsprache.

## Roadmap

Das Projekt ist abgeschlossen: Die Windows-App und der Android-Viewer sind fertig. Künftig sind nur noch kleine Korrekturen geplant, vor allem für seltene Layout-Fehler (Personen an ungünstigen Stellen in sehr großen Familien).

## Aus dem Quellcode bauen

Wenn du lieber lokal baust, brauchst du die Rust-Toolchain unter Windows:

```
cargo test --release
cargo build --release
```

Das Ergebnis ist `target/release/famtree.exe`. Das Projekt verwendet [egui/eframe](https://github.com/emilk/egui) für die Oberfläche.

Die Android-App (`android/`) wird ausschließlich vom GitHub-Actions-Workflow `.github/workflows/android.yml` gebaut (cargo-apk, Android SDK/NDK, eine kleine Java-Schicht und Signierung). Sie kompiliert den Desktop-Code aus `src/` (`model`, `container`, `layout`, `view`, `flags` und weitere), deshalb ist die Karte auf beiden Plattformen identisch. Um die APK mit einem eigenen Schlüssel zu signieren, lege die Repository-Secrets `ANDROID_KEYSTORE_BASE64`, `ANDROID_KEYSTORE_PASSWORD`, `ANDROID_KEY_ALIAS` und, falls abweichend, `ANDROID_KEY_PASSWORD` an; ohne sie verwendet der Workflow einen temporären Schlüssel.

## Fehlerberichte

Du hast ein Problem gefunden, vor allem bei der Platzierung der Personen auf der Karte? Bitte [eröffne ein Issue](https://github.com/john3sobieski/family-tree/issues) und hänge, wenn möglich, eine kleine `.famtree`-Datei an, die das Problem reproduziert (ohne echte personenbezogene Daten, falls der Baum privat ist). Bei der Android-App gib bitte auch Android-Version und Gerätemodell an.

## Lizenz

FamTree steht unter der [PolyForm Noncommercial License 1.0.0](https://polyformproject.org/licenses/noncommercial/1.0.0/). Siehe [`LICENSE.md`](LICENSE.md).

Kurz gesagt:

- Du darfst die App und ihren Quellcode für **nichtkommerzielle** Zwecke (private Nutzung, Familienforschung, Hobby, Bildung, Forschung und Ähnliches) verwenden, kopieren, ändern und weitergeben.
- **Verkauf oder kommerzielle Nutzung sind nicht erlaubt.**
- Wenn du die App oder eine geänderte Version weitergibst (Fork, Neubuild, Weiterverbreitung), musst du die Lizenzbedingungen und die Zeile `Required Notice` aus `LICENSE.md` beibehalten, die john3sobieski als Autor nennt.

Diese Zusammenfassung ist keine Rechtsberatung, maßgeblich ist der Lizenztext.
