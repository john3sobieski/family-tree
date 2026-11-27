//! Interface localization. Russian source strings are the keys; every other language is looked up in `TABLE`.
//! A missing key falls back to Russian.

use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::atomic::{AtomicU8, Ordering};
use std::sync::OnceLock;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Lang {
    Ru,
    En,
    Zh,
    Es,
    Fr,
    De,
    Pl,
}

impl Lang {
    pub const ALL: [Lang; 7] = [Lang::Ru, Lang::En, Lang::Zh, Lang::Es, Lang::Fr, Lang::De, Lang::Pl];

    pub fn code(self) -> &'static str {
        match self {
            Lang::Ru => "ru",
            Lang::En => "en",
            Lang::Zh => "zh",
            Lang::Es => "es",
            Lang::Fr => "fr",
            Lang::De => "de",
            Lang::Pl => "pl",
        }
    }

    /// Language name in the language itself (shown in the language menu).
    pub fn native_name(self) -> &'static str {
        match self {
            Lang::Ru => "Русский",
            Lang::En => "English",
            Lang::Zh => "中文",
            Lang::Es => "Español",
            Lang::Fr => "Français",
            Lang::De => "Deutsch",
            Lang::Pl => "Polski",
        }
    }

    pub fn from_code(s: &str) -> Option<Lang> {
        let s = s.trim().to_ascii_lowercase();
        Lang::ALL.into_iter().find(|l| l.code() == s)
    }

    /// Column in `TABLE`; Russian is the key itself.
    pub fn column(self) -> Option<usize> {
        match self {
            Lang::Ru => None,
            Lang::En => Some(0),
            Lang::Zh => Some(1),
            Lang::Es => Some(2),
            Lang::Fr => Some(3),
            Lang::De => Some(4),
            Lang::Pl => Some(5),
        }
    }

    fn from_u8(n: u8) -> Lang {
        Lang::ALL.get(n as usize).copied().unwrap_or(Lang::Ru)
    }
}

static CURRENT: AtomicU8 = AtomicU8::new(0);

pub fn current() -> Lang {
    Lang::from_u8(CURRENT.load(Ordering::Relaxed))
}

pub fn set(lang: Lang) {
    let n = Lang::ALL.iter().position(|l| *l == lang).unwrap_or(0) as u8;
    CURRENT.store(n, Ordering::Relaxed);
}

/// Translates a Russian key into the current language.
pub fn tr<'a>(key: &'a str) -> &'a str {
    translate(current(), key)
}

/// Like `tr`, then replaces `{name}` placeholders with the given values.
pub fn trf(key: &str, args: &[(&str, String)]) -> String {
    let mut s = translate(current(), key).to_string();
    for (name, value) in args {
        s = s.replace(&format!("{{{name}}}"), value);
    }
    s
}

pub fn translate<'a>(lang: Lang, key: &'a str) -> &'a str {
    let Some(col) = lang.column() else { return key };
    match table().get(key) {
        Some(row) => row[col],
        None => key,
    }
}

fn table() -> &'static HashMap<&'static str, [&'static str; 6]> {
    static MAP: OnceLock<HashMap<&'static str, [&'static str; 6]>> = OnceLock::new();
    MAP.get_or_init(|| TABLE.iter().copied().collect())
}

/// "35 лет", "35 years", "35岁": the unit depends on the language and on the number.
pub fn years_text(n: i32) -> String {
    years_text_for(current(), n)
}

fn years_text_for(lang: Lang, n: i32) -> String {
    let (a, b) = (n % 10, n % 100);
    match lang {
        Lang::Ru => {
            let w = if (11..=14).contains(&b) {
                "лет"
            } else if a == 1 {
                "год"
            } else if (2..=4).contains(&a) {
                "года"
            } else {
                "лет"
            };
            format!("{n} {w}")
        }
        Lang::En => format!("{n} {}", if n == 1 { "year" } else { "years" }),
        Lang::Es => format!("{n} {}", if n == 1 { "año" } else { "años" }),
        Lang::Fr => format!("{n} {}", if n < 2 { "an" } else { "ans" }),
        Lang::De => format!("{n} {}", if n == 1 { "Jahr" } else { "Jahre" }),
        Lang::Pl => {
            let w = if n == 1 {
                "rok"
            } else if (2..=4).contains(&a) && !(12..=14).contains(&b) {
                "lata"
            } else {
                "lat"
            };
            format!("{n} {w}")
        }
        Lang::Zh => format!("{n}岁"),
    }
}

// ---- language choice: saved in a small file, otherwise the system UI language ----

/// Folder for the app's small settings files (`language.txt`, `theme.txt`).
pub fn settings_dir() -> Option<PathBuf> {
    let base = std::env::var_os("APPDATA")
        .or_else(|| std::env::var_os("XDG_CONFIG_HOME"))
        .map(PathBuf::from)
        .or_else(|| std::env::var_os("HOME").map(|h| PathBuf::from(h).join(".config")))?;
    Some(base.join("FamTree"))
}

fn settings_file() -> Option<PathBuf> {
    Some(settings_dir()?.join("language.txt"))
}

pub fn save(lang: Lang) {
    if let Some(p) = settings_file() {
        if let Some(dir) = p.parent() {
            let _ = std::fs::create_dir_all(dir);
        }
        let _ = std::fs::write(p, lang.code());
    }
}

fn load_saved() -> Option<Lang> {
    Lang::from_code(&std::fs::read_to_string(settings_file()?).ok()?)
}

#[cfg(windows)]
fn detect_system() -> Lang {
    #[link(name = "kernel32")]
    extern "system" {
        fn GetUserDefaultUILanguage() -> u16;
    }
    // Primary language id = low 10 bits of the LANGID.
    let id = unsafe { GetUserDefaultUILanguage() } & 0x3FF;
    match id {
        0x19 => Lang::Ru,
        0x04 => Lang::Zh,
        0x0A => Lang::Es,
        0x0C => Lang::Fr,
        0x07 => Lang::De,
        0x15 => Lang::Pl,
        _ => Lang::En,
    }
}

#[cfg(not(windows))]
fn detect_system() -> Lang {
    let v = std::env::var("LC_ALL").or_else(|_| std::env::var("LANG")).unwrap_or_default();
    Lang::from_code(v.get(..2).unwrap_or("")).unwrap_or(Lang::En)
}

/// Call once at startup, before the window is created.
pub fn init() {
    set(load_saved().unwrap_or_else(detect_system));
}

/// (Russian key, [en, zh, es, fr, de, pl]).
const TABLE: &[(&str, [&str; 6])] = &[
    ("Область фото на карточке", ["Photo area for the card", "卡片照片区域", "Zona de la foto en la tarjeta", "Zone de la photo sur la carte", "Fotobereich für die Karte", "Obszar zdjęcia na karcie"]),
    ("Перетаскивайте рамку; масштаб — колесо мыши или ползунок", ["Drag the frame; zoom with the mouse wheel or the slider", "拖动方框；用鼠标滚轮或滑块缩放", "Arrastra el marco; zoom con la rueda del ratón o el control deslizante", "Faites glisser le cadre ; zoom avec la molette ou le curseur", "Rahmen ziehen; Zoom mit Mausrad oder Schieberegler", "Przeciągaj ramkę; powiększenie kółkiem myszy lub suwakiem"]),
    ("Карточка", ["Card", "卡片", "Tarjeta", "Carte", "Karte", "Karta"]),
    ("Масштаб", ["Zoom", "缩放", "Zoom", "Zoom", "Zoom", "Powiększenie"]),
    ("Сбросить", ["Reset", "重置", "Restablecer", "Réinitialiser", "Zurücksetzen", "Resetuj"]),
    ("Готово", ["Done", "完成", "Listo", "Terminé", "Fertig", "Gotowe"]),
    ("Семейное дерево", ["Family Tree", "家谱", "Árbol genealógico", "Arbre généalogique", "Stammbaum", "Drzewo genealogiczne"]),
    ("Новое дерево", ["New tree", "新家谱", "Árbol nuevo", "Nouvel arbre", "Neuer Stammbaum", "Nowe drzewo"]),
    ("Светлая тема", ["Light theme", "浅色主题", "Tema claro", "Thème clair", "Helles Design", "Jasny motyw"]),
    ("Тёмная тема", ["Dark theme", "深色主题", "Tema oscuro", "Thème sombre", "Dunkles Design", "Ciemny motyw"]),
    ("Язык", ["Language", "语言", "Idioma", "Langue", "Sprache", "Język"]),
    ("Не удалось открыть {path}: {err}", ["Could not open {path}: {err}", "无法打开 {path}：{err}", "No se pudo abrir {path}: {err}", "Impossible d’ouvrir {path} : {err}", "{path} konnte nicht geöffnet werden: {err}", "Nie udało się otworzyć {path}: {err}"]),
    ("Несохранённые изменения", ["Unsaved changes", "未保存的更改", "Cambios sin guardar", "Modifications non enregistrées", "Ungespeicherte Änderungen", "Niezapisane zmiany"]),
    ("В «{name}» есть несохранённые изменения. Сохранить их?", ["“{name}” has unsaved changes. Save them?", "“{name}”有未保存的更改。要保存吗？", "«{name}» tiene cambios sin guardar. ¿Guardarlos?", "« {name} » contient des modifications non enregistrées. Les enregistrer ?", "„{name}“ hat ungespeicherte Änderungen. Speichern?", "„{name}” ma niezapisane zmiany. Zapisać je?"]),
    ("Сохранить", ["Save", "保存", "Guardar", "Enregistrer", "Speichern", "Zapisz"]),
    ("Не сохранять", ["Don’t save", "不保存", "No guardar", "Ne pas enregistrer", "Nicht speichern", "Nie zapisuj"]),
    ("Отмена", ["Cancel", "取消", "Cancelar", "Annuler", "Abbrechen", "Anuluj"]),
    ("Не удалось сохранить: {err}", ["Could not save: {err}", "无法保存：{err}", "No se pudo guardar: {err}", "Échec de l’enregistrement : {err}", "Speichern fehlgeschlagen: {err}", "Nie udało się zapisać: {err}"]),
    ("семья", ["family", "家谱", "familia", "famille", "familie", "rodzina"]),
    ("Открыть", ["Open", "打开", "Abrir", "Ouvrir", "Öffnen", "Otwórz"]),
    ("Файл", ["File", "文件", "Archivo", "Fichier", "Datei", "Plik"]),
    ("Новое", ["New", "新建", "Nuevo", "Nouveau", "Neu", "Nowe"]),
    ("Сохранить как…", ["Save as…", "另存为…", "Guardar como…", "Enregistrer sous…", "Speichern unter…", "Zapisz jako…"]),
    ("Отвязать .famtree от приложения", ["Unlink .famtree from the app", "取消 .famtree 文件关联", "Desvincular .famtree de la aplicación", "Dissocier .famtree de l’application", ".famtree von der App trennen", "Odłącz .famtree od aplikacji"]),
    ("Открывать .famtree двойным кликом", ["Open .famtree with a double click", "双击打开 .famtree 文件", "Abrir .famtree con doble clic", "Ouvrir .famtree par double-clic", ".famtree per Doppelklick öffnen", "Otwieraj .famtree dwukrotnym kliknięciem"]),
    ("Ассоциация .famtree: {err}", [".famtree association: {err}", ".famtree 文件关联：{err}", "Asociación de .famtree: {err}", "Association .famtree : {err}", ".famtree-Zuordnung: {err}", "Skojarzenie .famtree: {err}"]),
    ("Просмотр", ["View", "查看", "Vista", "Consultation", "Ansicht", "Podgląd"]),
    ("Редакция", ["Edit", "编辑", "Edición", "Édition", "Bearbeiten", "Edycja"]),
    ("Список", ["List", "列表", "Lista", "Liste", "Liste", "Lista"]),
    ("По размеру", ["Fit to view", "适应窗口", "Ajustar", "Ajuster", "Einpassen", "Dopasuj"]),
    ("·  Людей: {total}  ·  на схеме: {shown}", ["·  People: {total}  ·  on chart: {shown}", "·  人数：{total}  ·  图中：{shown}", "·  Personas: {total}  ·  en el esquema: {shown}", "·  Personnes : {total}  ·  sur le schéma : {shown}", "·  Personen: {total}  ·  im Schema: {shown}", "·  Osób: {total}  ·  na schemacie: {shown}"]),
    ("·  Выбран: {name}", ["·  Selected: {name}", "·  已选择：{name}", "·  Seleccionado: {name}", "·  Sélectionné : {name}", "·  Ausgewählt: {name}", "·  Wybrano: {name}"]),
    ("●  не сохранено", ["●  not saved", "●  未保存", "●  sin guardar", "●  non enregistré", "●  nicht gespeichert", "●  niezapisano"]),
    ("(автосохранение заработает после «Сохранить как…»)", ["(autosave starts working after “Save as…”)", "（执行“另存为…”后才会自动保存）", "(el autoguardado funcionará tras «Guardar como…»)", "(l’enregistrement automatique démarre après « Enregistrer sous… »)", "(Autospeichern funktioniert erst nach „Speichern unter…“)", "(autozapis zacznie działać po „Zapisz jako…”)"]),
    ("🔍 Найти человека…", ["🔍 Find a person…", "🔍 查找人物…", "🔍 Buscar persona…", "🔍 Trouver une personne…", "🔍 Person suchen…", "🔍 Znajdź osobę…"]),
    ("Никого не найдено", ["No one found", "未找到任何人", "No se encontró a nadie", "Personne trouvé", "Niemand gefunden", "Nikogo nie znaleziono"]),
    ("Дерево пусто. Откройте файл или добавьте первого человека в режиме «Редакция».", ["The tree is empty. Open a file or add the first person in “Edit” mode.", "树为空。请打开文件，或在“编辑”模式下添加第一个人。", "El árbol está vacío. Abre un archivo o añade a la primera persona en el modo «Edición».", "L’arbre est vide. Ouvrez un fichier ou ajoutez la première personne en mode « Édition ».", "Der Baum ist leer. Öffnen Sie eine Datei oder fügen Sie im Modus „Bearbeiten“ die erste Person hinzu.", "Drzewo jest puste. Otwórz plik lub dodaj pierwszą osobę w trybie „Edycja”."]),
    ("Имя", ["First name", "名", "Nombre", "Prénom", "Vorname", "Imię"]),
    ("Фамилия", ["Last name", "姓", "Apellido", "Nom", "Nachname", "Nazwisko"]),
    ("Пол", ["Gender", "性别", "Sexo", "Sexe", "Geschlecht", "Płeć"]),
    ("Рождение", ["Birth", "出生", "Nacimiento", "Naissance", "Geburt", "Urodzenie"]),
    ("Смерть", ["Death", "逝世", "Fallecimiento", "Décès", "Tod", "Śmierć"]),
    ("Возраст", ["Age", "年龄", "Edad", "Âge", "Alter", "Wiek"]),
    ("Специальность", ["Profession", "职业", "Profesión", "Profession", "Beruf", "Zawód"]),
    ("Все", ["All", "全部", "Todos", "Tous", "Alle", "Wszyscy"]),
    ("Мужчины", ["Men", "男性", "Hombres", "Hommes", "Männer", "Mężczyźni"]),
    ("Женщины", ["Women", "女性", "Mujeres", "Femmes", "Frauen", "Kobiety"]),
    ("Пол не указан", ["Gender not specified", "未指定性别", "Sexo no indicado", "Sexe non indiqué", "Geschlecht unbekannt", "Płeć nieokreślona"]),
    ("Мужской", ["Male", "男", "Masculino", "Masculin", "Männlich", "Mężczyzna"]),
    ("Женский", ["Female", "女", "Femenino", "Féminin", "Weiblich", "Kobieta"]),
    ("Не указан", ["Not specified", "未指定", "No indicado", "Non indiqué", "Nicht angegeben", "Nie podano"]),
    ("Поиск:", ["Search:", "搜索：", "Buscar:", "Rechercher :", "Suche:", "Szukaj:"]),
    ("имя, специальность, заметки…", ["name, profession, notes…", "姓名、职业、备注…", "nombre, profesión, notas…", "nom, profession, notes…", "Name, Beruf, Notizen…", "imię, zawód, notatki…"]),
    ("Пол:", ["Gender:", "性别：", "Sexo:", "Sexe :", "Geschlecht:", "Płeć:"]),
    ("Очистить поиск", ["Clear search", "清除搜索", "Borrar búsqueda", "Effacer la recherche", "Suche löschen", "Wyczyść wyszukiwanie"]),
    ("Показано: {shown} из {total}", ["Showing {shown} of {total}", "显示 {shown}，共 {total}", "Mostrando {shown} de {total}", "{shown} affichés sur {total}", "{shown} von {total} angezeigt", "Pokazano {shown} z {total}"]),
    ("не удалось загрузить фото", ["couldn’t load the photo", "无法加载照片", "no se pudo cargar la foto", "impossible de charger la photo", "Foto konnte nicht geladen werden", "nie udało się wczytać zdjęcia"]),
    ("Фото: {n}", ["Photos: {n}", "照片：{n}", "Fotos: {n}", "Photos : {n}", "Fotos: {n}", "Zdjęcia: {n}"]),
    ("Без имени", ["Unnamed", "无名", "Sin nombre", "Sans nom", "Ohne Namen", "Bez imienia"]),
    ("Родился(ась)", ["Born", "出生日期", "Nacimiento", "Naissance", "Geboren", "Data urodzenia"]),
    ("Умер(ла)", ["Died", "逝世日期", "Fallecimiento", "Décès", "Gestorben", "Data śmierci"]),
    ("Место рождения", ["Place of birth", "出生地", "Lugar de nacimiento", "Lieu de naissance", "Geburtsort", "Miejsce urodzenia"]),
    ("Место смерти", ["Place of death", "逝世地", "Lugar de fallecimiento", "Lieu de décès", "Sterbeort", "Miejsce śmierci"]),
    ("Национальность", ["Nationality", "国籍", "Nacionalidad", "Nationalité", "Staatsangehörigkeit", "Narodowość"]),
    ("Родители", ["Parents", "父母", "Padres", "Parents", "Eltern", "Rodzice"]),
    ("Партнёры", ["Partners", "伴侣", "Parejas", "Partenaires", "Partner", "Partnerzy"]),
    ("Дети", ["Children", "子女", "Hijos", "Enfants", "Kinder", "Dzieci"]),
    ("Заметки", ["Notes", "备注", "Notas", "Notes", "Notizen", "Notatki"]),
    ("р. {year}", ["b. {year}", "生于 {year}", "n. {year}", "né(e) en {year}", "geb. {year}", "ur. {year}"]),
    ("ум. {year}", ["d. {year}", "卒于 {year}", "f. {year}", "décédé(e) en {year}", "gest. {year}", "zm. {year}"]),
    ("(прожил(а))", ["(at death)", "（享年）", "(al fallecer)", "(au décès)", "(bei Tod)", "(w chwili śmierci)"]),
    ("Люди", ["People", "人物", "Personas", "Personnes", "Personen", "Osoby"]),
    ("Поиск по имени…", ["Search by name…", "按姓名搜索…", "Buscar por nombre…", "Rechercher par nom…", "Nach Namen suchen…", "Szukaj po imieniu…"]),
    ("Новый человек", ["New person", "新人物", "Nueva persona", "Nouvelle personne", "Neue Person", "Nowa osoba"]),
    ("У человека уже есть два родителя", ["This person already has two parents", "此人已有两位父母", "Esta persona ya tiene dos progenitores", "Cette personne a déjà deux parents", "Diese Person hat bereits zwei Elternteile", "Ta osoba ma już dwoje rodziców"]),
    ("день", ["day", "日", "día", "jour", "Tag", "dzień"]),
    ("месяц", ["month", "月", "mes", "mois", "Monat", "miesiąc"]),
    ("год", ["year", "年", "año", "année", "Jahr", "rok"]),
    ("Изображения", ["Images", "图片", "Imágenes", "Images", "Bilder", "Obrazy"]),
    ("Убрать", ["Remove", "移除", "Quitar", "Retirer", "Entfernen", "Usuń"]),
    ("Добавить страну…", ["Add country…", "添加国家…", "Añadir país…", "Ajouter un pays…", "Land hinzufügen…", "Dodaj kraj…"]),
    ("Поиск…", ["Search…", "搜索…", "Buscar…", "Rechercher…", "Suchen…", "Szukaj…"]),
    ("Выберите человека в списке или кликните по карточке на схеме.", ["Select a person in the list or click a card on the chart.", "请在列表中选择人物，或点击图上的卡片。", "Elige a una persona en la lista o haz clic en una tarjeta del esquema.", "Choisissez une personne dans la liste ou cliquez sur une carte du schéma.", "Wähle eine Person in der Liste oder klicke auf eine Karte im Schema.", "Wybierz osobę z listy lub kliknij kartę na schemacie."]),
    ("Фамилия (сейчас)", ["Last name (current)", "现用姓", "Apellido (actual)", "Nom (actuel)", "Nachname (aktuell)", "Nazwisko (obecne)"]),
    ("Фамилия при рождении", ["Maiden name", "娘家姓", "Apellido de nacimiento", "Nom de naissance", "Geburtsname", "Nazwisko rodowe"]),
    ("если сменила после свадьбы", ["if changed after marriage", "如婚后更改", "si lo cambió tras la boda", "si changé après le mariage", "falls nach der Heirat geändert", "jeśli zmieniono po ślubie"]),
    ("Отчество", ["Patronymic", "父称", "Patronímico", "Patronyme", "Vatersname", "Patronimik"]),
    ("Главное фото (на карточке)", ["Main photo (on the card)", "主照片（显示在卡片上）", "Foto principal (en la tarjeta)", "Photo principale (sur la carte)", "Hauptfoto (auf der Karte)", "Zdjęcie główne (na karcie)"]),
    ("нет", ["none", "无", "ninguna", "aucune", "keins", "brak"]),
    ("Выбрать…", ["Choose…", "选择…", "Elegir…", "Choisir…", "Auswählen…", "Wybierz…"]),
    ("Фото в заметках: {n}", ["Photos in notes: {n}", "备注中的照片：{n}", "Fotos en las notas: {n}", "Photos dans les notes : {n}", "Fotos in den Notizen: {n}", "Zdjęcia w notatkach: {n}"]),
    ("Добавить…", ["Add…", "添加…", "Añadir…", "Ajouter…", "Hinzufügen…", "Dodaj…"]),
    ("Подпись…", ["Caption…", "说明…", "Pie de foto…", "Légende…", "Beschriftung…", "Podpis…"]),
    ("Сделать главным", ["Make main", "设为主照片", "Hacer principal", "Définir comme principale", "Als Hauptfoto", "Ustaw jako główne"]),
    ("Связи", ["Relationships", "关系", "Relaciones", "Relations", "Beziehungen", "Powiązania"]),
    ("Убрать связь", ["Remove link", "移除关系", "Quitar relación", "Supprimer le lien", "Beziehung entfernen", "Usuń powiązanie"]),
    ("Добавить нового человека:", ["Add a new person:", "添加新人物：", "Añadir una persona nueva:", "Ajouter une nouvelle personne :", "Neue Person hinzufügen:", "Dodaj nową osobę:"]),
    ("Родитель", ["Parent", "父母", "Padre/madre", "Parent", "Elternteil", "Rodzic"]),
    ("Партнёр", ["Partner", "伴侣", "Pareja", "Partenaire", "Partner", "Partner"]),
    ("Ребёнок", ["Child", "子女", "Hijo/a", "Enfant", "Kind", "Dziecko"]),
    ("Брат/сестра", ["Sibling", "兄弟姐妹", "Hermano/a", "Frère/sœur", "Geschwister", "Rodzeństwo"]),
    ("Связать с существующим человеком:", ["Link to an existing person:", "关联到已有人物：", "Vincular con una persona existente:", "Lier à une personne existante :", "Mit vorhandener Person verknüpfen:", "Powiąż z istniejącą osobą:"]),
    ("Начните вводить имя…", ["Start typing a name…", "开始输入姓名…", "Empieza a escribir un nombre…", "Commencez à saisir un nom…", "Namen eingeben…", "Zacznij wpisywać imię…"]),
    ("родитель", ["parent", "父母", "progenitor", "parent", "Elternteil", "rodzic"]),
    ("партнёр", ["partner", "伴侣", "pareja", "partenaire", "Partner", "partner"]),
    ("ребёнок", ["child", "子女", "hijo/a", "enfant", "Kind", "dziecko"]),
    ("Это «я» — схема строится от предков этого человека.", ["This is “me” — the chart is built from this person’s ancestors.", "这是“我”——图表从此人的祖先开始绘制。", "Este soy «yo»: el esquema se construye desde los antepasados de esta persona.", "C’est « moi » — le schéma part des ancêtres de cette personne.", "Das bin „ich“ – das Schema wird von den Vorfahren dieser Person aus aufgebaut.", "To „ja” — schemat budowany jest od przodków tej osoby."]),
    ("Это «я» (сделать корнем дерева)", ["This is “me” (make the tree root)", "这是“我”（设为家谱根节点）", "Este soy «yo» (hacer raíz del árbol)", "C’est « moi » (en faire la racine de l’arbre)", "Das bin „ich“ (als Wurzel festlegen)", "To „ja” (ustaw jako korzeń drzewa)"]),
    ("Удалить человека вместе со всеми его связями?", ["Delete this person along with all their links?", "删除此人及其所有关系？", "¿Eliminar a esta persona con todas sus relaciones?", "Supprimer cette personne avec tous ses liens ?", "Diese Person samt allen Beziehungen löschen?", "Usunąć tę osobę wraz ze wszystkimi powiązaniami?"]),
    ("Да, удалить", ["Yes, delete", "是，删除", "Sí, eliminar", "Oui, supprimer", "Ja, löschen", "Tak, usuń"]),
    ("Удалить человека…", ["Delete person…", "删除人物…", "Eliminar persona…", "Supprimer la personne…", "Person löschen…", "Usuń osobę…"]),
    ("Нельзя связать так", ["Can’t link this way", "无法这样关联", "No se puede vincular así", "Liaison impossible", "So lässt sich nicht verknüpfen", "Nie można tak powiązać"]),
    ("Новый родитель", ["New parent", "新父母", "Nuevo progenitor", "Nouveau parent", "Neuer Elternteil", "Nowy rodzic"]),
    ("Новый партнёр", ["New partner", "新伴侣", "Nueva pareja", "Nouveau partenaire", "Neuer Partner", "Nowy partner"]),
    ("Новый ребёнок", ["New child", "新子女", "Nuevo hijo", "Nouvel enfant", "Neues Kind", "Nowe dziecko"]),
    ("Новый брат/сестра", ["New sibling", "新兄弟姐妹", "Nuevo hermano/a", "Nouveau frère/sœur", "Neues Geschwister", "Nowe rodzeństwo"]),
    ("Чтобы добавить брата/сестру, сначала добавьте родителя", ["To add a sibling, add a parent first", "要添加兄弟姐妹，请先添加父母", "Para añadir un hermano/a, añade primero un progenitor", "Pour ajouter un frère ou une sœur, ajoutez d’abord un parent", "Um Geschwister hinzuzufügen, füge zuerst einen Elternteil hinzu", "Aby dodać rodzeństwo, najpierw dodaj rodzica"]),
    ("Человек {id} не найден", ["Person {id} not found", "未找到人物 {id}", "No se encontró a la persona {id}", "Personne {id} introuvable", "Person {id} nicht gefunden", "Nie znaleziono osoby {id}"]),
    ("Связь человека с самим собой", ["A person can’t be linked to themselves", "人物不能与自己建立关系", "Una persona no puede relacionarse consigo misma", "Une personne ne peut pas être liée à elle-même", "Eine Person kann nicht mit sich selbst verknüpft werden", "Osoba nie może być powiązana sama ze sobą"]),
    ("Такая связь уже есть", ["This link already exists", "该关系已存在", "Esa relación ya existe", "Ce lien existe déjà", "Diese Beziehung existiert bereits", "Takie powiązanie już istnieje"]),
    ("Связь создаёт цикл в родстве", ["The link would create a cycle in the family relations", "该关系会造成亲属关系循环", "La relación crearía un ciclo de parentesco", "Ce lien créerait un cycle de parenté", "Die Beziehung würde einen Verwandtschaftszyklus erzeugen", "To powiązanie utworzyłoby cykl w pokrewieństwie"]),
    ("Версия файла {v} новее, чем поддерживает приложение", ["File version {v} is newer than this app supports", "文件版本 {v} 比本应用支持的版本新", "La versión del archivo {v} es más reciente de lo que admite la aplicación", "La version {v} du fichier est plus récente que celle prise en charge par l’application", "Dateiversion {v} ist neuer, als die App unterstützt", "Wersja pliku {v} jest nowsza, niż obsługuje aplikacja"]),
    ("Ошибка данных: {err}", ["Data error: {err}", "数据错误：{err}", "Error de datos: {err}", "Erreur de données : {err}", "Datenfehler: {err}", "Błąd danych: {err}"]),
    ("Ошибка файла: {err}", ["File error: {err}", "文件错误：{err}", "Error de archivo: {err}", "Erreur de fichier : {err}", "Dateifehler: {err}", "Błąd pliku: {err}"]),
    ("Ошибка архива: {err}", ["Archive error: {err}", "压缩包错误：{err}", "Error del archivo comprimido: {err}", "Erreur d’archive : {err}", "Archivfehler: {err}", "Błąd archiwum: {err}"]),
    ("В файле нет записи {name}", ["The file has no entry {name}", "文件中没有条目 {name}", "El archivo no contiene la entrada {name}", "Le fichier ne contient pas l’entrée {name}", "Die Datei enthält den Eintrag {name} nicht", "W pliku brak wpisu {name}"]),
    ("Не найдено фото {hash}", ["Photo {hash} not found", "未找到照片 {hash}", "No se encontró la foto {hash}", "Photo {hash} introuvable", "Foto {hash} nicht gefunden", "Nie znaleziono zdjęcia {hash}"]),
    ("Ошибка чтения файла: {err}", ["File read error: {err}", "读取文件出错：{err}", "Error al leer el archivo: {err}", "Erreur de lecture du fichier : {err}", "Fehler beim Lesen der Datei: {err}", "Błąd odczytu pliku: {err}"]),
    ("Не удалось обработать фото: {err}", ["Could not process the photo: {err}", "无法处理照片：{err}", "No se pudo procesar la foto: {err}", "Impossible de traiter la photo : {err}", "Foto konnte nicht verarbeitet werden: {err}", "Nie udało się przetworzyć zdjęcia: {err}"]),
    ("Семейное дерево (.famtree)", ["Family Tree (.famtree)", "家谱 (.famtree)", "Árbol genealógico (.famtree)", "Arbre généalogique (.famtree)", "Stammbaum (.famtree)", "Drzewo genealogiczne (.famtree)"]),
    ("только Windows", ["Windows only", "仅限 Windows", "solo Windows", "Windows uniquement", "nur Windows", "tylko Windows"]),
    ("Экспорт…", ["Export…", "导出…", "Exportar…", "Exporter…", "Exportieren…", "Eksport…"]),
    ("Экспорт", ["Export", "导出", "Exportar", "Exporter", "Exportieren", "Eksport"]),
    ("Формат", ["Format", "格式", "Formato", "Format", "Format", "Format"]),
    ("Цвета", ["Colors", "颜色", "Colores", "Couleurs", "Farben", "Kolory"]),
    ("Тёмные", ["Dark", "深色", "Oscuros", "Sombres", "Dunkel", "Ciemne"]),
    ("Светлые", ["Light", "浅色", "Claros", "Clairs", "Hell", "Jasne"]),
    ("Фон", ["Background", "背景", "Fondo", "Arrière-plan", "Hintergrund", "Tło"]),
    ("Прозрачный", ["Transparent", "透明", "Transparente", "Transparent", "Transparent", "Przezroczyste"]),
    ("Белый", ["White", "白色", "Blanco", "Blanc", "Weiß", "Białe"]),
    ("Чёрный", ["Black", "黑色", "Negro", "Noir", "Schwarz", "Czarne"]),
    ("Экспортировать…", ["Export…", "导出…", "Exportar…", "Exporter…", "Exportieren…", "Eksportuj…"]),
    ("Закрыть", ["Close", "关闭", "Cerrar", "Fermer", "Schließen", "Zamknij"]),
    ("Назад", ["Back", "返回", "Atrás", "Retour", "Zurück", "Wstecz"]),
    ("Изображение PNG", ["PNG image", "PNG 图片", "Imagen PNG", "Image PNG", "PNG-Bild", "Obraz PNG"]),
    ("Документ PDF", ["PDF document", "PDF 文档", "Documento PDF", "Document PDF", "PDF-Dokument", "Dokument PDF"]),
    ("Экспорт… {percent}%", ["Exporting… {percent}%", "正在导出… {percent}%", "Exportando… {percent}%", "Export en cours… {percent} %", "Export läuft… {percent} %", "Eksport… {percent}%"]),
    ("Готово: {path} ({size})", ["Done: {path} ({size})", "完成：{path}（{size}）", "Listo: {path} ({size})", "Terminé : {path} ({size})", "Fertig: {path} ({size})", "Gotowe: {path} ({size})"]),
    ("Не удалось экспортировать: {err}", ["Export failed: {err}", "导出失败：{err}", "No se pudo exportar: {err}", "Échec de l’export : {err}", "Export fehlgeschlagen: {err}", "Nie udało się wyeksportować: {err}"]),
    ("Нечего экспортировать: дерево пустое.", ["Nothing to export: the tree is empty.", "没有可导出的内容：树为空。", "Nada que exportar: el árbol está vacío.", "Rien à exporter : l’arbre est vide.", "Nichts zu exportieren: Der Baum ist leer.", "Nie ma czego eksportować: drzewo jest puste."]),
    ("Дерево слишком большое для PNG, используйте PDF.", ["The tree is too large for PNG, use PDF.", "树太大，无法导出为 PNG，请使用 PDF。", "El árbol es demasiado grande para PNG, usa PDF.", "L’arbre est trop grand pour un PNG, utilisez le PDF.", "Der Baum ist für PNG zu groß, bitte PDF verwenden.", "Drzewo jest za duże na PNG, użyj PDF."]),
    ("Экспорт отменён.", ["Export cancelled.", "导出已取消。", "Exportación cancelada.", "Export annulé.", "Export abgebrochen.", "Eksport anulowany."]),
    ("Внутренняя ошибка при экспорте.", ["Internal error during export.", "导出时发生内部错误。", "Error interno durante la exportación.", "Erreur interne pendant l’export.", "Interner Fehler beim Export.", "Błąd wewnętrzny podczas eksportu."]),
    ("Не хватает памяти.", ["Not enough memory.", "内存不足。", "No hay memoria suficiente.", "Mémoire insuffisante.", "Nicht genug Speicher.", "Za mało pamięci."]),
    ("PNG: {w} × {h} px (масштаб ×{scale}). Всё дерево целиком, с отступами по краям.", ["PNG: {w} × {h} px (scale ×{scale}). The whole tree, with margins around the edges.", "PNG：{w} × {h} 像素（缩放 ×{scale}）。整棵树，四周留有边距。", "PNG: {w} × {h} px (escala ×{scale}). Todo el árbol, con márgenes en los bordes.", "PNG : {w} × {h} px (échelle ×{scale}). L’arbre entier, avec des marges sur les bords.", "PNG: {w} × {h} px (Maßstab ×{scale}). Der ganze Baum, mit Rand an den Seiten.", "PNG: {w} × {h} px (skala ×{scale}). Całe drzewo, z marginesami wokół."]),
    ("Дерево очень большое, детали будут мелкими. Для таких деревьев лучше PDF.", ["The tree is very large, details will be small. PDF is better for such trees.", "树非常大，细节会很小。这种情况更适合用 PDF。", "El árbol es muy grande y los detalles serán pequeños. Para estos árboles es mejor PDF.", "L’arbre est très grand, les détails seront petits. Le PDF convient mieux dans ce cas.", "Der Baum ist sehr groß, Details werden klein. Für solche Bäume ist PDF besser.", "Drzewo jest bardzo duże, szczegóły będą drobne. Dla takich drzew lepszy jest PDF."]),
    ("PDF: векторный, масштабируется без потери качества. Страница {w} × {h} pt.", ["PDF: vector, zooms without losing quality. Page {w} × {h} pt.", "PDF：矢量格式，缩放不失真。页面 {w} × {h} pt。", "PDF: vectorial, se amplía sin perder calidad. Página {w} × {h} pt.", "PDF : vectoriel, zoom sans perte de qualité. Page {w} × {h} pt.", "PDF: Vektor, zoomt ohne Qualitätsverlust. Seite {w} × {h} pt.", "PDF: wektorowy, powiększa się bez utraty jakości. Strona {w} × {h} pt."]),
    ("Основное", ["Main", "基本信息", "Datos básicos", "Informations", "Allgemeines", "Podstawowe"]),
    ("Даты и места", ["Dates and places", "日期与地点", "Fechas y lugares", "Dates et lieux", "Daten und Orte", "Daty i miejsca"]),
    ("Откройте файл .famtree из файлового менеджера (Открыть с помощью → FamTree).", ["Open a .famtree file from your file manager (Open with → FamTree).", "请在文件管理器中打开 .famtree 文件（打开方式 → FamTree）。", "Abre un archivo .famtree desde tu gestor de archivos (Abrir con → FamTree).", "Ouvrez un fichier .famtree depuis votre gestionnaire de fichiers (Ouvrir avec → FamTree).", "Öffne eine .famtree-Datei im Dateimanager (Öffnen mit → FamTree).", "Otwórz plik .famtree z menedżera plików (Otwórz za pomocą → FamTree)."]),
    ("Загрузка…", ["Loading…", "加载中…", "Cargando…", "Chargement…", "Wird geladen…", "Wczytywanie…"]),
    ("Не удалось открыть файл", ["Could not open the file", "无法打开文件", "No se pudo abrir el archivo", "Impossible d’ouvrir le fichier", "Datei konnte nicht geöffnet werden", "Nie udało się otworzyć pliku"]),
    ("Файл: {name}", ["File: {name}", "文件：{name}", "Archivo: {name}", "Fichier : {name}", "Datei: {name}", "Plik: {name}"]),
    ("Людей в дереве: {n}", ["People in the tree: {n}", "树中人数：{n}", "Personas en el árbol: {n}", "Personnes dans l’arbre : {n}", "Personen im Baum: {n}", "Osób w drzewie: {n}"]),
];

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::BTreeSet;

    fn placeholders(s: &str) -> BTreeSet<String> {
        let mut out = BTreeSet::new();
        let mut rest = s;
        while let Some(i) = rest.find('{') {
            let Some(j) = rest[i..].find('}') else { break };
            out.insert(rest[i + 1..i + j].to_string());
            rest = &rest[i + j + 1..];
        }
        out
    }

    #[test]
    fn table_is_consistent() {
        let mut seen = BTreeSet::new();
        for (key, row) in TABLE {
            assert!(seen.insert(*key), "duplicate key: {key}");
            for text in row {
                assert!(!text.trim().is_empty(), "empty translation for: {key}");
                assert_eq!(placeholders(text), placeholders(key), "placeholders differ for: {key}");
            }
        }
    }

    #[test]
    fn lookup_and_fallback() {
        assert_eq!(translate(Lang::Ru, "Открыть"), "Открыть");
        assert_eq!(translate(Lang::En, "Открыть"), "Open");
        assert_eq!(translate(Lang::Pl, "Открыть"), "Otwórz");
        assert_eq!(translate(Lang::De, "нет такого ключа"), "нет такого ключа");
    }

    #[test]
    fn language_codes_roundtrip() {
        for l in Lang::ALL {
            assert_eq!(Lang::from_code(l.code()), Some(l));
        }
        assert_eq!(Lang::from_code("xx"), None);
    }

    #[test]
    fn years_units() {
        assert_eq!(years_text_for(Lang::Ru, 21), "21 год");
        assert_eq!(years_text_for(Lang::Ru, 112), "112 лет");
        assert_eq!(years_text_for(Lang::En, 1), "1 year");
        assert_eq!(years_text_for(Lang::En, 35), "35 years");
        assert_eq!(years_text_for(Lang::Pl, 1), "1 rok");
        assert_eq!(years_text_for(Lang::Pl, 22), "22 lata");
        assert_eq!(years_text_for(Lang::Pl, 12), "12 lat");
        assert_eq!(years_text_for(Lang::Pl, 25), "25 lat");
        assert_eq!(years_text_for(Lang::Fr, 0), "0 an");
        assert_eq!(years_text_for(Lang::Zh, 35), "35岁");
    }

    /// Every `tr("...")` / `trf("...")` key used in the sources must exist in `TABLE`.
    #[test]
    fn all_used_keys_are_in_the_table() {
        let sources = [
            ("app.rs", include_str!("app.rs")),
            ("export.rs", include_str!("export.rs")),
            ("chrome.rs", include_str!("chrome.rs")),
            ("edit.rs", include_str!("edit.rs")),
            ("list.rs", include_str!("list.rs")),
            ("model.rs", include_str!("model.rs")),
            ("container.rs", include_str!("container.rs")),
            ("crop.rs", include_str!("crop.rs")),
            ("media.rs", include_str!("media.rs")),
            ("assoc.rs", include_str!("assoc.rs")),
            ("person_window.rs", include_str!("person_window.rs")),
            ("search.rs", include_str!("search.rs")),
            ("view.rs", include_str!("view.rs")),
            ("main.rs", include_str!("main.rs")),
        ];
        let known: BTreeSet<&str> = TABLE.iter().map(|(k, _)| *k).collect();
        let mut missing = Vec::new();
        for (file, src) in sources {
            for pat in ["tr(\"", "trf(\"", "placeholder_name(\""] {
                for (i, _) in src.match_indices(pat) {
                    let prev = src[..i].chars().last().unwrap_or(' ');
                    if prev.is_alphanumeric() || prev == '_' {
                        continue;
                    }
                    let start = i + pat.len();
                    let mut end = start;
                    let bytes = src.as_bytes();
                    while end < bytes.len() && !(bytes[end] == b'"' && bytes[end - 1] != b'\\') {
                        end += 1;
                    }
                    let key = &src[start..end];
                    if !known.contains(key) {
                        missing.push(format!("{file}: {key}"));
                    }
                }
            }
        }
        assert!(missing.is_empty(), "keys missing from TABLE: {missing:?}");
    }
}
