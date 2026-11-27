**English** | [简体中文](L10n/zh-cn/README_zh-CN.md) | [Español](L10n/es/README_es.md) | [Português (BR)](L10n/pt-BR/README_pt-BR.md) | [日本語](L10n/ja/README_ja.md) | [Русский](L10n/ru/README_ru.md) | [Français](L10n/fr/README_fr.md) | [Deutsch](L10n/de/README_de.md) | [한국어](L10n/ko/README_ko.md) | [Türkçe](L10n/tr/README.tr.md) | [Bahasa Indonesia](L10n/id/README.id.md)

# FamTree

Created and maintained by **[john3sobieski](https://github.com/john3sobieski)**. Repository: <https://github.com/john3sobieski/family-tree>

A desktop app for Windows for building, editing and browsing your family tree, plus an Android app for viewing it on your phone. Everything lives in a single `.famtree` file: people, relationships and all photos.

> **Status:** ready to use. The project is considered finished: from here on only minor fixes are expected. Download the Windows installer or the Android APK from [Releases](https://github.com/john3sobieski/family-tree/releases) (see [Installation](#installation)).

## Features

- **Three modes**
  - **View** – read-only mind map with smooth zoom and pan.
  - **Edit** – add and change people, photos and relationships.
  - **List** – a sortable, filterable table of everyone in the tree.
- **Mind map built around you.** Mark one person as "me" and the tree is drawn from your top ancestors downwards, so parents, siblings, uncles, aunts, cousins and more distant relatives all show up automatically. Both sides of the family are laid out next to each other, and the in-laws' families are included too.
- **Photo cards.** Each card shows a passport-style photo, the name, the surname (with the maiden name in brackets for women), the patronymic, a colour strip for gender and up to two flags of nationality.
- **Person window.** Click a card to open a larger window with the full photo, a gallery (10+ photos per person are fine), dates, age, places of birth and death, profession, notes and clickable links to parents, partners and children.
- **Search.** Type a name and the camera flies to that person on the map and highlights them. In List mode it works as a filter. `Ctrl+F` focuses the search field.
- **Relationships.** Parents, partners, children and siblings can be added in one click or linked to people who already exist. Cycles and duplicate links are rejected.
- **Incomplete dates.** Know only the year? That's fine, any part of a date can be left empty.
- **Nationalities.** Pick up to two per person from a built-in list of 197 countries (all UN members plus the Vatican, Palestine, Taiwan and Kosovo), with a search box. Flags are drawn by the app itself.
- **Built for big trees.** Only visible cards are drawn, thumbnails are loaded lazily, and the layout is recalculated only when the structure changes. It is tested with trees of thousands of people.
- **Autosave.** Once the tree has a file, every change is saved automatically a moment after you stop typing. For a new tree, use **Save as…** first. You are also asked before closing if something could not be saved.
- **Interface languages.** Русский, English, 中文, Español, Français, Deutsch and Polski. Use the **Language** menu to switch; the choice is remembered. On the first launch the app follows the Windows display language and falls back to English. Country names are translated as well.
- **Export.** **File → Export…** saves the whole tree as a PNG or a vector PDF, with margins around the edges. For PNG you choose the background (transparent, white or black) and dark or light colors; the size is picked automatically so that everything stays sharp (up to ×2). The PDF can be zoomed without losing quality. Very large trees are better exported to PDF.
- **Double-click to open.** In the **File** menu you can associate `.famtree` files with the app.
- **Dark and light theme.** Switch with the sun/moon button at the right end of the status bar; the choice is remembered.
- **Android app (view only).** Open the same `.famtree` file on your phone or tablet. The map looks exactly as on the PC (same layout, cards, flags and lines) and supports drag and pinch-zoom, search with a camera flight to the person, a full-screen person page with photos and links to relatives, **Fit to view** and a light/dark theme. The file is never modified: editing, List mode and export stay on the PC. The interface follows the phone's language (the same seven languages, English otherwise). See [Installation → Android](#android).

## Installation

### Windows

The app runs on Windows 10 and Windows 11. It is the same installer and the same exe for both.

1. Open the [Releases](https://github.com/john3sobieski/family-tree/releases) page and download **`FamTree-Setup-<version>.exe`**.
2. Run it. It installs for your user only, so no administrator rights are needed. The wizard offers to open `.famtree` files with FamTree and to add a desktop shortcut.
3. Start **FamTree** from the Start menu.

Notes:

- Windows may show a SmartScreen warning because the installer and the `.exe` are not signed. Choose *More info → Run anyway*.
- To update, run a newer Setup over the old version. Your trees and settings are kept.
- To uninstall, use *Settings → Apps*. Your `.famtree` files are never deleted.

### Android

The Android app only shows trees and never changes your file. It needs Android 10 or newer on a 64-bit ARM device (practically all current phones and tablets).

1. Download **`FamTree-<version>.apk`** from the [Releases](https://github.com/john3sobieski/family-tree/releases) page directly onto your device.
2. Open the file. Android will ask you once to allow installing apps from this source (your browser or file manager); confirm.
3. Put your `.famtree` file on the device (cloud storage, messenger, USB cable). Then either open it from a file manager with **Open with → FamTree**, or start FamTree and tap **Open** to pick the file. The last opened file is remembered. If you open another `.famtree` while the app is running, it is picked up without a restart.

Notes:

- The app is distributed only as an APK, not through Google Play. Play Protect may warn about an app from an unknown developer; choose to install anyway.
- To be able to reopen it, the app keeps a private copy of the last opened file; the original is never touched.

### Building it yourself

You can also get the installer from GitHub Actions without installing any tools (a GitHub account is needed): fork the [repository](https://github.com/john3sobieski/family-tree), open **Actions → Build → Run workflow**, and download the **`famtree-setup`** artifact from the finished run (kept for 90 days by default).

The Android APK is built the same way: **Actions → Android → Run workflow**, then download the **`famtree-android-<version>`** artifact.

## Quick start

1. Start `famtree.exe`. Use **File → New** to start your own tree, or **File → Open** to open an existing one.
2. Switch to **Edit** mode and add the first person (**＋ New person**). In an empty tree they automatically become "me".
3. Select a person and use **＋ Parent**, **＋ Partner**, **＋ Child** or **＋ Sibling** to grow the tree. Fill in names, dates, photos and notes on the right.
4. Choose **Save as…** and pick a name. From then on, changes are saved automatically.
5. Switch to **View** to explore the map: drag to pan, scroll to zoom, **Fit to view** to see everything, click a card for details.

**On Android:** open a `.famtree` file (see [Installation → Android](#android)), drag to pan and pinch with two fingers to zoom. **Fit to view** shows everything at once, the magnifier opens the person search, and tapping a card opens the person's page (tap a relative to jump to them). The system Back button or gesture closes the page or the search step by step; on the map it sends the app to the background. The sun/moon button switches the theme (by default the app follows the system theme).

## The `.famtree` file

A `.famtree` file is an ordinary ZIP archive:

```
tree.json            people and relationships (plain, human-readable text)
media/<hash>.jpg     photos, resized to at most 1600 px
thumb/<hash>.jpg     small thumbnails for cards
```

- Photos are stored inside the file, so you can send one `.famtree` to a relative and nothing is lost. Identical photos are stored only once.
- `tree.json` is readable in any text editor and has comments describing each relationship. If you want to edit it by hand, unpack the archive, edit the file and pack it back. Editing it directly inside the `.famtree` will break the archive.
- Older files keep opening in newer versions of the app.

## Known limitations

- Interface translations other than Russian have not been reviewed by native speakers yet. Corrections are welcome via issues.
- Dates are always shown as `DD.MM.YYYY`, whatever the language.
- No undo yet.
- Importing a large batch of photos at once can freeze the window for a moment.
- HEIC photos are not supported (JPEG/JFIF, PNG, WebP and BMP are).
- The thumbnail on a card is a centered crop, and you can't choose the crop area yet.
- In very large families the layout can still place people in odd spots or cross lines between branches (see the roadmap).
- Nationalities are not shown in List mode.
- Android: view only (no editing, List mode or export), 64-bit ARM devices with Android 10 or newer, APK only (not on Google Play).
- Android: photos can't be enlarged with a pinch gesture yet, and there is no language menu: the app follows the system language.

## Roadmap

The project is finished: the Windows app and the Android viewer are complete. From here on only minor fixes are planned, mainly for rare layout glitches (people placed in odd spots in very large families).

## Building from source

If you prefer to build locally, you need the Rust toolchain on Windows:

```
cargo test --release
cargo build --release
```

The result is `target/release/famtree.exe`. The project uses [egui/eframe](https://github.com/emilk/egui) for the interface.

The Android app (`android/`) is built only by the GitHub Actions workflow `.github/workflows/android.yml` (cargo-apk, Android SDK/NDK, a small Java layer and signing). It compiles the desktop code from `src/` (`model`, `container`, `layout`, `view`, `flags` and others), which is why the map is identical on both platforms. To sign the APK with your own key, add the repository secrets `ANDROID_KEYSTORE_BASE64`, `ANDROID_KEYSTORE_PASSWORD`, `ANDROID_KEY_ALIAS` and, if it differs, `ANDROID_KEY_PASSWORD`; without them the workflow uses a temporary key.

## Bug reports

Found a problem, especially with how people are placed on the map? Please [open an issue](https://github.com/john3sobieski/family-tree/issues) and, if you can, attach a small `.famtree` file that reproduces it (without real personal data, if the tree is private). For the Android app, please also mention your Android version and device model.

## License

FamTree is licensed under the [PolyForm Noncommercial License 1.0.0](https://polyformproject.org/licenses/noncommercial/1.0.0/). See [`LICENSE.md`](LICENSE.md).

In short:

- You may use, copy, modify and share the app and its source code for **noncommercial** purposes (personal use, family history, hobby, education, research and the like).
- **Selling it or using it commercially is not allowed.**
- Whenever you share the app or a modified version (a fork, a rebuild, a redistribution), you must keep the license terms and the `Required Notice` line from `LICENSE.md`, which credits john3sobieski as the author.

This summary is not legal advice, and the license text is what counts.
