//! Импорт фото: декодирование, поворот по EXIF, ресайз, миниатюра, sha256-хэш.
//! Результат кладётся в `MediaStore` (полное фото + миниатюра для карточки).

use crate::container::{MediaKind, MediaStore};
use crate::i18n::trf;
use image::codecs::jpeg::JpegEncoder;
use image::imageops::FilterType;
use image::{DynamicImage, ImageDecoder, ImageReader};
use sha2::{Digest, Sha256};
use std::fmt;
use std::io::Cursor;
use std::path::Path;

/// Максимальная сторона полного фото (px). Меньшие не увеличиваются.
pub const MAX_FULL_SIDE: u32 = 1600;
/// Миниатюра карточки: формат паспортного фото 3:4.
pub const THUMB_W: u32 = 200;
pub const THUMB_H: u32 = 267;
pub const JPEG_QUALITY: u8 = 85;
/// Соотношение сторон кадра главного фото (ширина / высота) — как у миниатюры карточки.
pub const CARD_ASPECT: f32 = THUMB_W as f32 / THUMB_H as f32;
/// Максимальное приближение в редакторе кадра (1.0 = самый большой кадр, влезающий в снимок).
pub const MAX_ZOOM: f32 = 8.0;

#[derive(Debug)]
pub enum MediaError {
    Io(String),
    Image(String),
}

impl fmt::Display for MediaError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            MediaError::Io(e) => write!(f, "{}", trf("Ошибка чтения файла: {err}", &[("err", e.to_string())])),
            MediaError::Image(e) => write!(f, "{}", trf("Не удалось обработать фото: {err}", &[("err", e.to_string())])),
        }
    }
}

impl std::error::Error for MediaError {}

impl From<std::io::Error> for MediaError {
    fn from(e: std::io::Error) -> Self {
        MediaError::Io(e.to_string())
    }
}
impl From<image::ImageError> for MediaError {
    fn from(e: image::ImageError) -> Self {
        MediaError::Image(e.to_string())
    }
}

/// sha256 исходных байт (hex) — идентификатор фото; одинаковые файлы не дублируются.
pub fn hash_bytes(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
}

/// Декодирует изображение и применяет поворот по EXIF.
pub fn decode(bytes: &[u8]) -> Result<DynamicImage, MediaError> {
    let reader = ImageReader::new(Cursor::new(bytes)).with_guessed_format()?;
    let mut decoder = reader.into_decoder()?;
    let orientation = decoder.orientation()?;
    let mut img = DynamicImage::from_decoder(decoder)?;
    img.apply_orientation(orientation);
    Ok(img)
}

fn encode_jpeg(img: &DynamicImage) -> Result<Vec<u8>, MediaError> {
    // JPEG без альфа-канала.
    let rgb = DynamicImage::ImageRgb8(img.to_rgb8());
    let mut buf = Vec::new();
    rgb.write_with_encoder(JpegEncoder::new_with_quality(&mut buf, JPEG_QUALITY))?;
    Ok(buf)
}

/// Импортирует фото из байт. Возвращает hash. Если такое фото уже есть — ничего не делает.
pub fn import_photo(store: &mut MediaStore, bytes: &[u8]) -> Result<String, MediaError> {
    let hash = hash_bytes(bytes);
    if store.has(MediaKind::Full, &hash) {
        return Ok(hash);
    }
    let img = decode(bytes)?;

    let full = if img.width() > MAX_FULL_SIDE || img.height() > MAX_FULL_SIDE {
        img.resize(MAX_FULL_SIDE, MAX_FULL_SIDE, FilterType::Lanczos3)
    } else {
        img.clone()
    };
    // Миниатюра: центральная обрезка под 3:4.
    let thumb = img.resize_to_fill(THUMB_W, THUMB_H, FilterType::Lanczos3);

    store.add(MediaKind::Full, &hash, encode_jpeg(&full)?);
    store.add(MediaKind::Thumb, &hash, encode_jpeg(&thumb)?);
    Ok(hash)
}

/// Область снимка в пикселях исходного изображения.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct CropRect {
    pub x: f32,
    pub y: f32,
    pub w: f32,
    pub h: f32,
}

/// Самый большой кадр 3:4, который помещается в снимок (при zoom = 1).
fn base_size(img_w: f32, img_h: f32) -> (f32, f32) {
    if img_w / img_h >= CARD_ASPECT {
        (img_h * CARD_ASPECT, img_h)
    } else {
        (img_w, img_w / CARD_ASPECT)
    }
}

/// Кадр для заданного приближения с центром в `(cx, cy)`; центр сдвигается, чтобы кадр не вылезал за снимок.
pub fn crop_for(img_w: f32, img_h: f32, zoom: f32, cx: f32, cy: f32) -> CropRect {
    let (bw, bh) = base_size(img_w, img_h);
    let zoom = zoom.clamp(1.0, MAX_ZOOM);
    let (w, h) = (bw / zoom, bh / zoom);
    CropRect {
        x: (cx - w / 2.0).clamp(0.0, (img_w - w).max(0.0)),
        y: (cy - h / 2.0).clamp(0.0, (img_h - h).max(0.0)),
        w,
        h,
    }
}

/// Сохраняет как отдельное фото только выбранный фрагмент: полное (≤ `MAX_FULL_SIDE`) + миниатюра 3:4.
/// Исходный снимок не сохраняется. Hash = sha256 готового JPEG, одинаковый фрагмент не дублируется.
pub fn import_cropped(store: &mut MediaStore, img: &DynamicImage, crop: CropRect) -> Result<String, MediaError> {
    let (iw, ih) = (img.width(), img.height());
    let x = (crop.x.round().max(0.0) as u32).min(iw.saturating_sub(1));
    let y = (crop.y.round().max(0.0) as u32).min(ih.saturating_sub(1));
    let w = (crop.w.round() as u32).max(1).min(iw - x);
    let h = (crop.h.round() as u32).max(1).min(ih - y);
    let part = img.crop_imm(x, y, w, h);
    let full = if w > MAX_FULL_SIDE || h > MAX_FULL_SIDE {
        part.resize(MAX_FULL_SIDE, MAX_FULL_SIDE, FilterType::Lanczos3)
    } else {
        part
    };
    let full_bytes = encode_jpeg(&full)?;
    let hash = hash_bytes(&full_bytes);
    if store.has(MediaKind::Full, &hash) {
        return Ok(hash);
    }
    let thumb = full.resize_exact(THUMB_W, THUMB_H, FilterType::Lanczos3);
    store.add(MediaKind::Full, &hash, full_bytes);
    store.add(MediaKind::Thumb, &hash, encode_jpeg(&thumb)?);
    Ok(hash)
}

pub fn import_photo_file(store: &mut MediaStore, path: &Path) -> Result<String, MediaError> {
    let bytes = std::fs::read(path)?;
    import_photo(store, &bytes)
}

#[cfg(test)]
mod tests {
    use super::*;
    use image::{ImageFormat, Rgb, RgbImage};

    fn png_bytes(w: u32, h: u32) -> Vec<u8> {
        let img = RgbImage::from_fn(w, h, |x, y| Rgb([(x % 256) as u8, (y % 256) as u8, 128]));
        let mut out = Vec::new();
        DynamicImage::ImageRgb8(img)
            .write_to(&mut Cursor::new(&mut out), ImageFormat::Png)
            .unwrap();
        out
    }

    fn dims(bytes: &[u8]) -> (u32, u32) {
        let img = image::load_from_memory(bytes).unwrap();
        (img.width(), img.height())
    }

    #[test]
    fn large_photo_is_downscaled_and_thumb_is_3x4() {
        let mut store = MediaStore::new();
        let hash = import_photo(&mut store, &png_bytes(3000, 2000)).unwrap();
        assert_eq!(hash.len(), 64);

        let (w, h) = dims(&store.get(MediaKind::Full, &hash).unwrap());
        assert_eq!(w, MAX_FULL_SIDE);
        assert!(h >= 1066 && h <= 1067, "h = {h}");

        let (tw, th) = dims(&store.get(MediaKind::Thumb, &hash).unwrap());
        assert_eq!((tw, th), (THUMB_W, THUMB_H));
    }

    #[test]
    fn small_photo_is_not_upscaled() {
        let mut store = MediaStore::new();
        let hash = import_photo(&mut store, &png_bytes(400, 300)).unwrap();
        assert_eq!(dims(&store.get(MediaKind::Full, &hash).unwrap()), (400, 300));
    }

    #[test]
    fn same_bytes_give_same_hash_and_dedupe() {
        let bytes = png_bytes(300, 300);
        let mut store = MediaStore::new();
        let h1 = import_photo(&mut store, &bytes).unwrap();
        let h2 = import_photo(&mut store, &bytes).unwrap();
        assert_eq!(h1, h2);
        assert_eq!(h1, hash_bytes(&bytes));
    }

    #[test]
    fn garbage_is_error() {
        let mut store = MediaStore::new();
        let res = import_photo(&mut store, b"not an image");
        assert!(matches!(res, Err(MediaError::Image(_)) | Err(MediaError::Io(_))));
    }

    #[test]
    fn hash_is_known_sha256() {
        // sha256("abc")
        assert_eq!(
            hash_bytes(b"abc"),
            "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad"
        );
    }

    #[test]
    fn crop_for_stays_inside_and_keeps_3x4() {
        // 16:9: кадр на всю высоту, центр прижимается к краю.
        let c = crop_for(1600.0, 900.0, 1.0, 0.0, 0.0);
        assert!((c.h - 900.0).abs() < 0.01 && c.y == 0.0 && c.x == 0.0);
        assert!((c.w / c.h - CARD_ASPECT).abs() < 0.001);
        let c = crop_for(1600.0, 900.0, 2.0, 5000.0, 5000.0);
        assert!((c.h - 450.0).abs() < 0.01);
        assert!((c.x + c.w - 1600.0).abs() < 0.01 && (c.y + c.h - 900.0).abs() < 0.01);
        // Вертикальный снимок: кадр на всю ширину.
        let c = crop_for(600.0, 1200.0, 1.0, 300.0, 600.0);
        assert!((c.w - 600.0).abs() < 0.01);
        // Zoom за пределами диапазона не ломает кадр.
        let c = crop_for(1000.0, 1000.0, 100.0, 500.0, 500.0);
        assert!((c.h - 1000.0 / MAX_ZOOM).abs() < 0.01);
    }

    #[test]
    fn import_cropped_takes_only_selected_part() {
        // Левая половина красная, правая синяя; берём кадр из правой части.
        let img = DynamicImage::ImageRgb8(RgbImage::from_fn(1600, 900, |x, _| {
            if x < 800 { Rgb([255, 0, 0]) } else { Rgb([0, 0, 255]) }
        }));
        let mut store = MediaStore::new();
        let crop = crop_for(1600.0, 900.0, 2.0, 1200.0, 450.0);
        let hash = import_cropped(&mut store, &img, crop).unwrap();

        let thumb = image::load_from_memory(&store.get(MediaKind::Thumb, &hash).unwrap()).unwrap().to_rgb8();
        assert_eq!((thumb.width(), thumb.height()), (THUMB_W, THUMB_H));
        let px = thumb.get_pixel(THUMB_W / 2, THUMB_H / 2);
        assert!(px[2] > 200 && px[0] < 60, "px = {px:?}");

        let full = image::load_from_memory(&store.get(MediaKind::Full, &hash).unwrap()).unwrap();
        assert!((full.width() as f32 / full.height() as f32 - CARD_ASPECT).abs() < 0.01);
        // Тот же фрагмент — тот же hash.
        assert_eq!(import_cropped(&mut store, &img, crop).unwrap(), hash);
    }
}
