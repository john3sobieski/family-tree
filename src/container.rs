//! Контейнер `.famtree`: ZIP-архив.
//!   tree.json            — данные (см. `model`)
//!   media/<hash>.jpg     — полные фото
//!   thumb/<hash>.jpg     — миниатюры для карточек
//!
//! `tree.json` лежит без сжатия: его текст виден в любом редакторе (только читать, не сохранять).
//! Сохранение атомарное: пишем во временный файл, затем переименовываем.
//! Память ограничена: фото копируются из старого файла по одному.

use crate::model::{ModelError, Tree};
use std::collections::{BTreeSet, HashMap, HashSet};
use std::fmt;
use std::fs::File;
use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use zip::write::SimpleFileOptions;
use zip::{CompressionMethod, ZipArchive, ZipWriter};

pub const TREE_ENTRY: &str = "tree.json";
const MEDIA_DIR: &str = "media/";
const THUMB_DIR: &str = "thumb/";
const EXT: &str = ".jpg";

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum MediaKind {
    Full,
    Thumb,
}

pub type MediaKey = (MediaKind, String);

fn entry_name(kind: MediaKind, hash: &str) -> String {
    match kind {
        MediaKind::Full => format!("{MEDIA_DIR}{hash}{EXT}"),
        MediaKind::Thumb => format!("{THUMB_DIR}{hash}{EXT}"),
    }
}

fn parse_entry_name(name: &str) -> Option<MediaKey> {
    let (kind, rest) = if let Some(r) = name.strip_prefix(MEDIA_DIR) {
        (MediaKind::Full, r)
    } else if let Some(r) = name.strip_prefix(THUMB_DIR) {
        (MediaKind::Thumb, r)
    } else {
        return None;
    };
    let hash = rest.strip_suffix(EXT)?;
    if hash.is_empty() {
        return None;
    }
    Some((kind, hash.to_string()))
}

#[derive(Debug)]
pub enum ContainerError {
    Io(String),
    Zip(String),
    Model(ModelError),
    MissingEntry(String),
    MissingMedia(String),
}

impl fmt::Display for ContainerError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ContainerError::Io(e) => write!(f, "Ошибка файла: {e}"),
            ContainerError::Zip(e) => write!(f, "Ошибка архива: {e}"),
            ContainerError::Model(e) => write!(f, "{e}"),
            ContainerError::MissingEntry(n) => write!(f, "В файле нет записи {n}"),
            ContainerError::MissingMedia(h) => write!(f, "Не найдено фото {h}"),
        }
    }
}

impl std::error::Error for ContainerError {}

impl From<std::io::Error> for ContainerError {
    fn from(e: std::io::Error) -> Self {
        ContainerError::Io(e.to_string())
    }
}
impl From<zip::result::ZipError> for ContainerError {
    fn from(e: zip::result::ZipError) -> Self {
        ContainerError::Zip(e.to_string())
    }
}
impl From<ModelError> for ContainerError {
    fn from(e: ModelError) -> Self {
        ContainerError::Model(e)
    }
}

/// Все hash фото, на которые ссылаются люди.
pub fn referenced_hashes(tree: &Tree) -> BTreeSet<String> {
    let mut set = BTreeSet::new();
    for p in tree.persons.values() {
        if let Some(h) = &p.main_photo {
            set.insert(h.clone());
        }
        for ph in &p.photos {
            set.insert(ph.hash.clone());
        }
    }
    set
}

/// Хранилище фото: уже лежащие в файле + новые (ещё не сохранённые).
#[derive(Default)]
pub struct MediaStore {
    path: Option<PathBuf>,
    existing: HashSet<MediaKey>,
    pending: HashMap<MediaKey, Vec<u8>>,
}

impl MediaStore {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn has(&self, kind: MediaKind, hash: &str) -> bool {
        let key = (kind, hash.to_string());
        self.pending.contains_key(&key) || self.existing.contains(&key)
    }

    /// Добавляет новое фото (попадёт в файл при следующем сохранении).
    pub fn add(&mut self, kind: MediaKind, hash: &str, bytes: Vec<u8>) {
        self.pending.insert((kind, hash.to_string()), bytes);
    }

    /// Читает фото (из памяти или из файла).
    pub fn get(&self, kind: MediaKind, hash: &str) -> Result<Vec<u8>, ContainerError> {
        let mut old = self.open_old();
        self.fetch(&mut old, kind, hash)?
            .ok_or_else(|| ContainerError::MissingMedia(hash.to_string()))
    }

    fn open_old(&self) -> Option<ZipArchive<File>> {
        let p = self.path.as_ref()?;
        let f = File::open(p).ok()?;
        ZipArchive::new(f).ok()
    }

    fn fetch(
        &self,
        old: &mut Option<ZipArchive<File>>,
        kind: MediaKind,
        hash: &str,
    ) -> Result<Option<Vec<u8>>, ContainerError> {
        let key = (kind, hash.to_string());
        if let Some(b) = self.pending.get(&key) {
            return Ok(Some(b.clone()));
        }
        if !self.existing.contains(&key) {
            return Ok(None);
        }
        match old {
            Some(ar) => Ok(Some(read_entry(ar, &entry_name(kind, hash))?)),
            None => Ok(None),
        }
    }
}

fn read_entry(ar: &mut ZipArchive<File>, name: &str) -> Result<Vec<u8>, ContainerError> {
    let mut f = ar
        .by_name(name)
        .map_err(|_| ContainerError::MissingEntry(name.to_string()))?;
    let mut v = Vec::with_capacity(f.size() as usize);
    f.read_to_end(&mut v)?;
    Ok(v)
}

/// Открывает `.famtree`: возвращает дерево и хранилище фото (фото не грузятся в память).
pub fn load(path: &Path) -> Result<(Tree, MediaStore), ContainerError> {
    let mut ar = ZipArchive::new(File::open(path)?)?;
    let bytes = read_entry(&mut ar, TREE_ENTRY)?;
    let json = String::from_utf8(bytes).map_err(|e| ContainerError::Io(e.to_string()))?;
    let tree = Tree::from_json(&json)?;
    let existing: HashSet<MediaKey> = ar.file_names().filter_map(parse_entry_name).collect();
    let store = MediaStore {
        path: Some(path.to_path_buf()),
        existing,
        pending: HashMap::new(),
    };
    Ok((tree, store))
}

/// Сохраняет дерево и используемые фото. Неиспользуемые фото не попадают в файл.
pub fn save(path: &Path, tree: &Tree, store: &mut MediaStore) -> Result<(), ContainerError> {
    let mut tmp_name = path.as_os_str().to_owned();
    tmp_name.push(".tmp");
    let tmp = PathBuf::from(tmp_name);

    let written = match write_archive(&tmp, tree, store) {
        Ok(w) => w,
        Err(e) => {
            let _ = std::fs::remove_file(&tmp);
            return Err(e);
        }
    };
    // Старый архив уже закрыт (на Windows нельзя заменить открытый файл).
    if let Err(e) = std::fs::rename(&tmp, path) {
        let _ = std::fs::remove_file(&tmp);
        return Err(e.into());
    }
    store.path = Some(path.to_path_buf());
    store.existing = written;
    store.pending.clear();
    Ok(())
}

fn write_archive(
    tmp: &Path,
    tree: &Tree,
    store: &MediaStore,
) -> Result<HashSet<MediaKey>, ContainerError> {
    let mut old = store.open_old();
    let mut zip = ZipWriter::new(File::create(tmp)?);

    // tree.json и JPEG храним без сжатия: JPEG уже сжат, а JSON остаётся читаемым текстом.
    // Старые файлы со сжатым tree.json по-прежнему открываются.
    let stored = SimpleFileOptions::default().compression_method(CompressionMethod::Stored);
    let json = tree.to_json()?;
    zip.start_file(TREE_ENTRY, stored.clone())?;
    zip.write_all(json.as_bytes())?;

    let mut written: HashSet<MediaKey> = HashSet::new();
    for hash in referenced_hashes(tree) {
        for kind in [MediaKind::Full, MediaKind::Thumb] {
            match store.fetch(&mut old, kind, &hash)? {
                Some(bytes) => {
                    zip.start_file(entry_name(kind, &hash), stored.clone())?;
                    zip.write_all(&bytes)?;
                    written.insert((kind, hash.clone()));
                }
                None if kind == MediaKind::Full => {
                    return Err(ContainerError::MissingMedia(hash));
                }
                None => {} // миниатюры может не быть
            }
        }
    }
    let file = zip.finish()?;
    file.sync_all()?;
    Ok(written)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::{Person, Photo};
    use std::time::{SystemTime, UNIX_EPOCH};

    fn temp_path(name: &str) -> PathBuf {
        let nanos = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_nanos();
        let mut p = std::env::temp_dir();
        p.push(format!("famtree_test_{}_{}_{}.famtree", name, std::process::id(), nanos));
        p
    }

    fn tree_with_photos(hashes: &[&str]) -> Tree {
        let mut t = Tree::new();
        let mut p = Person::new("Иван", "Петров");
        p.main_photo = Some(hashes[0].to_string());
        for h in &hashes[1..] {
            p.photos.push(Photo { hash: h.to_string(), caption: String::new() });
        }
        t.add_person(p);
        t
    }

    #[test]
    fn roundtrip_with_media() {
        let path = temp_path("roundtrip");
        let tree = tree_with_photos(&["aaa", "bbb"]);
        let mut store = MediaStore::new();
        store.add(MediaKind::Full, "aaa", vec![1, 2, 3]);
        store.add(MediaKind::Thumb, "aaa", vec![9]);
        store.add(MediaKind::Full, "bbb", vec![4, 5]);
        save(&path, &tree, &mut store).unwrap();

        let (t2, s2) = load(&path).unwrap();
        assert_eq!(tree, t2);
        assert_eq!(s2.get(MediaKind::Full, "aaa").unwrap(), vec![1, 2, 3]);
        assert_eq!(s2.get(MediaKind::Thumb, "aaa").unwrap(), vec![9]);
        assert_eq!(s2.get(MediaKind::Full, "bbb").unwrap(), vec![4, 5]);
        assert!(!s2.has(MediaKind::Thumb, "bbb"));
        let _ = std::fs::remove_file(&path);
    }

    #[test]
    fn resave_keeps_old_and_adds_new_media() {
        let path = temp_path("resave");
        let tree = tree_with_photos(&["aaa"]);
        let mut store = MediaStore::new();
        store.add(MediaKind::Full, "aaa", vec![1, 1, 1]);
        save(&path, &tree, &mut store).unwrap();

        let (mut t2, mut s2) = load(&path).unwrap();
        let id = *t2.persons.keys().next().unwrap();
        t2.persons.get_mut(&id).unwrap().photos.push(Photo { hash: "ccc".into(), caption: "".into() });
        s2.add(MediaKind::Full, "ccc", vec![7, 7]);
        save(&path, &t2, &mut s2).unwrap();

        let (_t3, s3) = load(&path).unwrap();
        assert_eq!(s3.get(MediaKind::Full, "aaa").unwrap(), vec![1, 1, 1]);
        assert_eq!(s3.get(MediaKind::Full, "ccc").unwrap(), vec![7, 7]);
        let _ = std::fs::remove_file(&path);
    }

    #[test]
    fn unreferenced_media_is_dropped() {
        let path = temp_path("gc");
        let tree = tree_with_photos(&["aaa"]);
        let mut store = MediaStore::new();
        store.add(MediaKind::Full, "aaa", vec![1]);
        store.add(MediaKind::Full, "zzz", vec![2]);
        save(&path, &tree, &mut store).unwrap();
        let (_t, s) = load(&path).unwrap();
        assert!(s.has(MediaKind::Full, "aaa"));
        assert!(!s.has(MediaKind::Full, "zzz"));
        let _ = std::fs::remove_file(&path);
    }

    #[test]
    fn missing_full_media_is_error_and_leaves_no_tmp() {
        let path = temp_path("missing");
        let tree = tree_with_photos(&["aaa"]);
        let mut store = MediaStore::new();
        let res = save(&path, &tree, &mut store);
        assert!(matches!(res, Err(ContainerError::MissingMedia(_))));
        assert!(!path.exists());
        let mut tmp = path.as_os_str().to_owned();
        tmp.push(".tmp");
        assert!(!PathBuf::from(tmp).exists());
    }

    #[test]
    fn save_leaves_no_tmp_file() {
        let path = temp_path("notmp");
        let tree = Tree::new();
        let mut store = MediaStore::new();
        save(&path, &tree, &mut store).unwrap();
        let mut tmp = path.as_os_str().to_owned();
        tmp.push(".tmp");
        assert!(path.exists());
        assert!(!PathBuf::from(tmp).exists());
        let _ = std::fs::remove_file(&path);
    }

    #[test]
    fn archive_without_tree_json_is_error() {
        let path = temp_path("notree");
        {
            let mut zip = ZipWriter::new(File::create(&path).unwrap());
            zip.start_file("x.txt", SimpleFileOptions::default()).unwrap();
            zip.write_all(b"hi").unwrap();
            zip.finish().unwrap();
        }
        let res = load(&path);
        assert!(matches!(res, Err(ContainerError::MissingEntry(_))));
        let _ = std::fs::remove_file(&path);
    }

    #[test]
    fn tree_json_is_stored_as_plain_text() {
        let path = temp_path("plain");
        let tree = tree_with_photos(&["aaa"]);
        let mut store = MediaStore::new();
        store.add(MediaKind::Full, "aaa", vec![1, 2, 3]);
        save(&path, &tree, &mut store).unwrap();

        let raw = std::fs::read(&path).unwrap();
        let needle = "\"first_name\": \"Иван\"".as_bytes();
        assert!(raw.windows(needle.len()).any(|w| w == needle));

        let mut ar = ZipArchive::new(File::open(&path).unwrap()).unwrap();
        assert_eq!(ar.by_name(TREE_ENTRY).unwrap().compression(), CompressionMethod::Stored);
        let _ = std::fs::remove_file(&path);
    }

    #[test]
    fn old_deflated_tree_json_still_loads() {
        let path = temp_path("olddeflate");
        {
            let mut zip = ZipWriter::new(File::create(&path).unwrap());
            let opts = SimpleFileOptions::default().compression_method(CompressionMethod::Deflated);
            zip.start_file(TREE_ENTRY, opts).unwrap();
            zip.write_all(br#"{"version":1,"persons":{"1":{"id":1,"first_name":"X"}}}"#).unwrap();
            zip.finish().unwrap();
        }
        let (t, _s) = load(&path).unwrap();
        assert_eq!(t.persons[&1].first_name, "X");
        let _ = std::fs::remove_file(&path);
    }

    #[test]
    fn entry_names_roundtrip() {
        let n = entry_name(MediaKind::Thumb, "abc");
        assert_eq!(n, "thumb/abc.jpg");
        assert_eq!(parse_entry_name(&n), Some((MediaKind::Thumb, "abc".to_string())));
        assert_eq!(parse_entry_name("tree.json"), None);
        assert_eq!(parse_entry_name("media/.jpg"), None);
    }
}
