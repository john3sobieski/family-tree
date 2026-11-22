[English](README.md) | [简体中文](README.zh-CN.md) | [Español](README.es.md) | [Português (BR)](README.pt-BR.md) | [日本語](README.ja.md) | [Русский](README.ru.md) | [Français](README.fr.md) | **Deutsch** | [한국어](README.ko.md) | [Türkçe](README.tr.md) | [Bahasa Indonesia](README.id.md)

# FamTree

Erstellt und gepflegt von **[john3sobieski](https://github.com/john3sobieski)**. Repository: <https://github.com/john3sobieski/family-tree>

Eine Desktop-Anwendung für Windows, mit der du deinen Stammbaum erstellen, bearbeiten und durchstöbern kannst. Alles steckt in einer einzigen `.famtree`-Datei: Personen, Beziehungen und alle Fotos.

> **Status:** frühe Entwicklung. Es gibt noch keine fertigen Releases, und die App ist derzeit nur verfügbar, wenn man sie über GitHub Actions selbst baut (siehe [Installation](#installation)).

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
- **Doppelklick zum Öffnen.** Im Menü **Datei** kannst du `.famtree`-Dateien mit der App verknüpfen.

## Installation

Die App läuft unter Windows 10 und Windows 11 (dieselbe exe) und wird von GitHub Actions aus dem Quellcode gebaut. Du brauchst also weder Rust noch andere Werkzeuge auf deinem Rechner. Du brauchst allerdings ein GitHub-Konto.

1. Erstelle einen **Fork** des [Repositorys](https://github.com/john3sobieski/family-tree) (oder nutze deine eigene Kopie).
2. Öffne in deinem Fork den Tab **Actions**. Falls GitHub danach fragt, aktiviere die Workflows für den Fork.
3. Wähle den Workflow **Build**, klicke auf **Run workflow** und warte, bis er fertig ist (etwa 2–3 Minuten). Er läuft außerdem bei jedem Push.
4. Öffne den abgeschlossenen Lauf und lade das Artefakt **`famtree-windows`** im Abschnitt *Artifacts* ganz unten herunter.
5. Entpacke es. Darin liegt `famtree.exe`. Es gibt nichts zu installieren, einfach starten.

Hinweise:

- Windows zeigt möglicherweise eine SmartScreen-Warnung an, weil die `.exe` nicht signiert ist. Wähle *Weitere Informationen → Trotzdem ausführen*, wenn du deinem eigenen Build vertraust.
- GitHub bewahrt Artefakte nur begrenzt auf (standardmäßig 90 Tage). Speichere die `.exe` daher an einem dauerhaften Ort.
- Wenn du `famtree.exe` verschiebst, klicke im Menü **Datei** erneut auf die Schaltfläche für die `.famtree`-Verknüpfung, weil die Verknüpfung auf den Speicherort der exe zeigt.

Sobald es einen richtigen Installer gibt, wird er unter **Releases** veröffentlicht, und dieser Abschnitt wird ersetzt.

## Schnellstart

1. Starte `famtree.exe`. Mit **Datei → Demo** siehst du einen Beispielbaum, mit **Datei → Neu** beginnst du deinen eigenen.
2. Wechsle in den Modus **Bearbeiten** und füge die erste Person hinzu (**＋ Neue Person**). In einem leeren Baum wird sie automatisch zu „ich“.
3. Wähle eine Person aus und nutze **＋ Elternteil**, **＋ Partner**, **＋ Kind** oder **＋ Geschwister**, um den Baum wachsen zu lassen. Namen, Daten, Fotos und Notizen trägst du rechts ein.
4. Wähle **Speichern unter…** und vergib einen Namen. Ab dann werden Änderungen automatisch gespeichert.
5. Wechsle zu **Ansicht**, um die Karte zu erkunden: Ziehen zum Verschieben, Scrollen zum Zoomen, **Einpassen**, um alles zu sehen, und ein Klick auf eine Karte für Details.

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
- Noch kein Rückgängig und kein Export nach PNG/PDF.
- Der Import vieler Fotos auf einmal kann das Fenster kurz einfrieren lassen.
- HEIC-Fotos werden nicht unterstützt (JPEG, PNG, WebP und BMP schon).
- Das Vorschaubild auf einer Karte ist ein zentrierter Ausschnitt, und den Ausschnitt kann man noch nicht selbst wählen.
- In sehr großen Familien kann das Layout Personen immer noch an merkwürdigen Stellen platzieren oder Linien zwischen Zweigen kreuzen (siehe Roadmap).
- Staatsangehörigkeiten werden im Listenmodus nicht angezeigt.

## Roadmap

Geplante Verbesserungen, in beliebiger Reihenfolge:

- [ ] **Ein richtiger Installer** statt einer nackten `.exe` aus Actions, veröffentlicht unter Releases.
- [ ] **Behebung von Layout-Fehlern**: Gelegentlich landen Personen an der falschen Stelle auf der Karte.

## Aus dem Quellcode bauen

Wenn du lieber lokal baust, brauchst du die Rust-Toolchain unter Windows:

```
cargo test --release
cargo build --release
```

Das Ergebnis ist `target/release/famtree.exe`. Das Projekt verwendet [egui/eframe](https://github.com/emilk/egui) für die Oberfläche.

## Fehlerberichte

Du hast ein Problem gefunden, vor allem bei der Platzierung der Personen auf der Karte? Bitte [eröffne ein Issue](https://github.com/john3sobieski/family-tree/issues) und hänge, wenn möglich, eine kleine `.famtree`-Datei an, die das Problem reproduziert (ohne echte personenbezogene Daten, falls der Baum privat ist).

## Lizenz

FamTree steht unter der [PolyForm Noncommercial License 1.0.0](https://polyformproject.org/licenses/noncommercial/1.0.0/). Siehe [`LICENSE.md`](LICENSE.md).

Kurz gesagt:

- Du darfst die App und ihren Quellcode für **nichtkommerzielle** Zwecke (private Nutzung, Familienforschung, Hobby, Bildung, Forschung und Ähnliches) verwenden, kopieren, ändern und weitergeben.
- **Verkauf oder kommerzielle Nutzung sind nicht erlaubt.**
- Wenn du die App oder eine geänderte Version weitergibst (Fork, Neubuild, Weiterverbreitung), musst du die Lizenzbedingungen und die Zeile `Required Notice` aus `LICENSE.md` beibehalten, die john3sobieski als Autor nennt.

Diese Zusammenfassung ist keine Rechtsberatung, maßgeblich ist der Lizenztext.
