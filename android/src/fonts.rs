// System CJK font (egui bundles none): needed for the Chinese UI and for Chinese/Japanese/Korean names in a tree.
use crate::model::Tree;
use eframe::egui;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::OnceLock;

/// Font bytes (leaked once: they live as long as the app) and the face index inside a .ttc.
static CJK: OnceLock<Option<(&'static [u8], u32)>> = OnceLock::new();
static APPLIED: AtomicBool = AtomicBool::new(false);

const DIRS: [&str; 4] = ["/system/fonts", "/product/fonts", "/system_ext/fonts", "/vendor/fonts"];
const NAMES: [&str; 6] = [
    "NotoSansCJK-Regular.ttc",
    "NotoSansSC-Regular.otf",
    "NotoSansSC-VF.ttf",
    "SourceHanSansCN-Regular.otf",
    "DroidSansFallback.ttf",
    "DroidSansFallbackFull.ttf",
];

fn is_cjk(c: char) -> bool {
    matches!(c as u32, 0x2E80..=0x9FFF | 0xAC00..=0xD7AF | 0xF900..=0xFAFF | 0xFF00..=0xFFEF | 0x20000..=0x2FA1F)
}

/// True if any text in the tree has CJK characters (the JSON form contains every string of the tree).
pub fn tree_has_cjk(tree: &Tree) -> bool {
    serde_json::to_string(tree).map_or(false, |s| s.chars().any(is_cjk))
}

fn find() -> Option<PathBuf> {
    for name in NAMES {
        for dir in DIRS {
            let p = Path::new(dir).join(name);
            if p.is_file() {
                return Some(p);
            }
        }
    }
    // Other vendors name it differently: take the first CJK-looking font file.
    for dir in DIRS {
        let Ok(rd) = std::fs::read_dir(dir) else { continue };
        let mut found: Vec<PathBuf> = rd
            .filter_map(|e| e.ok())
            .map(|e| e.path())
            .filter(|p| {
                let n = p.file_name().and_then(|n| n.to_str()).unwrap_or("").to_ascii_lowercase();
                let font = n.ends_with(".ttc") || n.ends_with(".otf") || n.ends_with(".ttf");
                font && (n.contains("cjk") || n.contains("notosanssc") || n.contains("sourcehan"))
            })
            .collect();
        found.sort();
        if let Some(p) = found.into_iter().next() {
            return Some(p);
        }
    }
    None
}

/// Face with Simplified Chinese in a Noto CJK collection (order: JP, KR, SC, TC, HK); 0 for a single font.
fn face_index(bytes: &[u8]) -> u32 {
    if bytes.len() > 12 && &bytes[..4] == b"ttcf" {
        let faces = u32::from_be_bytes([bytes[8], bytes[9], bytes[10], bytes[11]]);
        if faces >= 3 {
            return 2;
        }
    }
    0
}

fn load() -> Option<(&'static [u8], u32)> {
    let path = find()?;
    let bytes = std::fs::read(&path).ok()?;
    let index = face_index(&bytes);
    log::info!("CJK font: {} (face {index})", path.display());
    Some((Box::leak(bytes.into_boxed_slice()), index))
}

/// Reads the font file (up to ~20 MB): call from a background thread so the UI does not stall.
pub fn prefetch() {
    CJK.get_or_init(load);
}

/// Adds the font as a fallback for all text. Once per run.
pub fn ensure(ctx: &egui::Context) {
    if APPLIED.swap(true, Ordering::SeqCst) {
        return;
    }
    let Some(&(bytes, index)) = CJK.get_or_init(load).as_ref() else {
        log::warn!("no system CJK font found");
        return;
    };
    let mut data = egui::FontData::from_static(bytes);
    data.index = index;
    let mut fonts = egui::FontDefinitions::default();
    fonts.font_data.insert("cjk".to_owned(), data);
    for family in [egui::FontFamily::Proportional, egui::FontFamily::Monospace] {
        if let Some(list) = fonts.families.get_mut(&family) {
            list.push("cjk".to_owned());
        }
    }
    ctx.set_fonts(fonts);
    ctx.request_repaint();
}
