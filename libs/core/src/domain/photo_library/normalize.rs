use std::io::Cursor;

use chrono::{DateTime, FixedOffset, NaiveDate, Utc};
use image::{
    DynamicImage, ImageFormat, ImageReader, Limits, RgbImage, codecs::jpeg::JpegEncoder,
    imageops::FilterType, metadata::Orientation,
};
use sha2::{Digest, Sha256};

use crate::domain::photo_library::error::{PhotoLibraryError, PhotoLibraryOutcome};

pub const MAX_SIDE: u32 = 2400;
pub const THUMBNAIL_SIDE: u32 = 400;
pub const STORED_MIME_TYPE: &str = "image/jpeg";

const IMAGE_QUALITY: u8 = 85;
const THUMBNAIL_QUALITY: u8 = 75;
const MAX_DECODED_SIDE: u32 = 20_000;
const MAX_DECODED_BYTES: u64 = 768 * 1024 * 1024;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NormalizedImage {
    pub bytes: Vec<u8>,
    pub thumbnail: Vec<u8>,
    pub width: u32,
    pub height: u32,
    pub content_hash: String,
    pub taken_at: Option<DateTime<Utc>>,
}

pub fn normalize_image(input: &[u8]) -> PhotoLibraryOutcome<NormalizedImage> {
    if input.is_empty() {
        return Err(PhotoLibraryError::EmptyImage);
    }
    let format = image::guess_format(input).map_err(|_| PhotoLibraryError::UnsupportedImage)?;
    if !matches!(
        format,
        ImageFormat::Jpeg | ImageFormat::Png | ImageFormat::WebP
    ) {
        return Err(PhotoLibraryError::UnsupportedImage);
    }

    if format == ImageFormat::Jpeg && !jpeg_is_complete(input) {
        return Err(PhotoLibraryError::CorruptImage);
    }

    let metadata = read_metadata(input);

    let mut reader = ImageReader::with_format(Cursor::new(input), format);
    let mut limits = Limits::default();
    limits.max_image_width = Some(MAX_DECODED_SIDE);
    limits.max_image_height = Some(MAX_DECODED_SIDE);
    limits.max_alloc = Some(MAX_DECODED_BYTES);
    reader.limits(limits);
    let mut decoded = reader
        .decode()
        .map_err(|_| PhotoLibraryError::CorruptImage)?;

    if let Some(orientation) = metadata.orientation {
        decoded.apply_orientation(orientation);
    }

    let flattened = DynamicImage::ImageRgb8(flatten_on_white(decoded));
    let full = fit_within(flattened, MAX_SIDE, FilterType::Lanczos3);
    let thumbnail = fit_within(full.clone(), THUMBNAIL_SIDE, FilterType::Triangle);

    let full = full.into_rgb8();
    let bytes = encode_jpeg(&full, IMAGE_QUALITY)?;
    let thumbnail = encode_jpeg(&thumbnail.into_rgb8(), THUMBNAIL_QUALITY)?;

    Ok(NormalizedImage {
        content_hash: hex_digest(&bytes),
        width: full.width(),
        height: full.height(),
        bytes,
        thumbnail,
        taken_at: metadata.taken_at,
    })
}

fn jpeg_is_complete(bytes: &[u8]) -> bool {
    let mut at = 2;
    while at + 1 < bytes.len() {
        if bytes[at] != 0xFF {
            at += 1;
            continue;
        }
        match bytes[at + 1] {
            0xD9 => return true,
            0xFF => at += 1,
            0x00 | 0x01 | 0xD0..=0xD8 => at += 2,
            _ => {
                let Some(length) = bytes.get(at + 2..at + 4) else {
                    return false;
                };
                at += 2 + usize::from(u16::from_be_bytes([length[0], length[1]]));
            }
        }
    }
    false
}

#[derive(Default)]
struct ExifMetadata {
    orientation: Option<Orientation>,
    taken_at: Option<DateTime<Utc>>,
}

fn read_metadata(input: &[u8]) -> ExifMetadata {
    let Ok(exif) = exif::Reader::new().read_from_container(&mut Cursor::new(input)) else {
        return ExifMetadata::default();
    };

    let orientation = exif
        .get_field(exif::Tag::Orientation, exif::In::PRIMARY)
        .and_then(|field| field.value.get_uint(0))
        .and_then(|value| u8::try_from(value).ok())
        .and_then(Orientation::from_exif);

    let taken_at = exif
        .get_field(exif::Tag::DateTimeOriginal, exif::In::PRIMARY)
        .and_then(|field| match &field.value {
            exif::Value::Ascii(parts) => parts.first(),
            _ => None,
        })
        .and_then(|ascii| {
            let mut stamp = exif::DateTime::from_ascii(ascii).ok()?;
            if let Some(offset) = exif
                .get_field(exif::Tag::OffsetTimeOriginal, exif::In::PRIMARY)
                .and_then(|field| match &field.value {
                    exif::Value::Ascii(parts) => parts.first(),
                    _ => None,
                })
            {
                let _ = stamp.parse_offset(offset);
            }
            to_utc(&stamp)
        });

    ExifMetadata {
        orientation,
        taken_at,
    }
}

fn to_utc(stamp: &exif::DateTime) -> Option<DateTime<Utc>> {
    let local = NaiveDate::from_ymd_opt(
        i32::from(stamp.year),
        u32::from(stamp.month),
        u32::from(stamp.day),
    )?
    .and_hms_opt(
        u32::from(stamp.hour),
        u32::from(stamp.minute),
        u32::from(stamp.second),
    )?;
    let offset = match stamp.offset {
        Some(minutes) => FixedOffset::east_opt(i32::from(minutes) * 60)?,
        None => FixedOffset::east_opt(0)?,
    };
    local
        .and_local_timezone(offset)
        .single()
        .map(|moment| moment.with_timezone(&Utc))
}

fn flatten_on_white(image: DynamicImage) -> RgbImage {
    if !image.color().has_alpha() {
        return image.into_rgb8();
    }
    let rgba = image.into_rgba8();
    RgbImage::from_fn(rgba.width(), rgba.height(), |x, y| {
        let pixel = rgba.get_pixel(x, y);
        let alpha = u32::from(pixel[3]);
        let blend = |channel: u8| {
            let value = (u32::from(channel) * alpha + 255 * (255 - alpha) + 127) / 255;
            u8::try_from(value).unwrap_or(u8::MAX)
        };
        image::Rgb([blend(pixel[0]), blend(pixel[1]), blend(pixel[2])])
    })
}

fn fit_within(image: DynamicImage, side: u32, filter: FilterType) -> DynamicImage {
    if image.width().max(image.height()) > side {
        image.resize(side, side, filter)
    } else {
        image
    }
}

fn encode_jpeg(image: &RgbImage, quality: u8) -> PhotoLibraryOutcome<Vec<u8>> {
    let mut out = Vec::new();
    JpegEncoder::new_with_quality(&mut out, quality)
        .encode_image(image)
        .map_err(|_| PhotoLibraryError::CorruptImage)?;
    Ok(out)
}

pub fn hex_digest(bytes: &[u8]) -> String {
    Sha256::digest(bytes)
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect()
}

#[cfg(test)]
mod tests {
    use image::{ImageEncoder, Rgb, Rgba, RgbaImage, codecs::png::PngEncoder};

    use super::*;

    fn gradient(width: u32, height: u32) -> RgbImage {
        RgbImage::from_fn(width, height, |x, y| {
            Rgb([(x % 256) as u8, (y % 256) as u8, ((x + y) % 256) as u8])
        })
    }

    fn jpeg_of(image: &RgbImage) -> Vec<u8> {
        encode_jpeg(image, 90).unwrap()
    }

    fn png_of(image: &RgbaImage) -> Vec<u8> {
        let mut out = Vec::new();
        PngEncoder::new(&mut out)
            .write_image(
                image.as_raw(),
                image.width(),
                image.height(),
                image::ExtendedColorType::Rgba8,
            )
            .unwrap();
        out
    }

    fn with_exif(jpeg: &[u8], fields: Vec<exif::Field>) -> Vec<u8> {
        let mut writer = exif::experimental::Writer::new();
        for field in &fields {
            writer.push_field(field);
        }
        let mut tiff = Cursor::new(Vec::new());
        writer.write(&mut tiff, false).unwrap();
        let tiff = tiff.into_inner();

        let mut segment = b"Exif\0\0".to_vec();
        segment.extend_from_slice(&tiff);
        let length = u16::try_from(segment.len() + 2).unwrap();

        let mut out = jpeg[..2].to_vec();
        out.extend_from_slice(&[0xFF, 0xE1]);
        out.extend_from_slice(&length.to_be_bytes());
        out.extend_from_slice(&segment);
        out.extend_from_slice(&jpeg[2..]);
        out
    }

    fn field(tag: exif::Tag, value: exif::Value) -> exif::Field {
        exif::Field {
            tag,
            ifd_num: exif::In::PRIMARY,
            value,
        }
    }

    fn exif_of(bytes: &[u8]) -> Option<exif::Exif> {
        exif::Reader::new()
            .read_from_container(&mut Cursor::new(bytes))
            .ok()
    }

    #[test]
    fn a_large_image_is_reduced_to_2400_and_its_thumbnail_to_400() {
        let source = jpeg_of(&gradient(5000, 2500));

        let normalized = normalize_image(&source).unwrap();

        assert_eq!((normalized.width, normalized.height), (2400, 1200));
        let thumbnail = image::load_from_memory(&normalized.thumbnail).unwrap();
        assert_eq!((thumbnail.width(), thumbnail.height()), (400, 200));
        let stored = image::load_from_memory(&normalized.bytes).unwrap();
        assert_eq!((stored.width(), stored.height()), (2400, 1200));
    }

    #[test]
    fn a_small_image_is_never_enlarged() {
        let source = jpeg_of(&gradient(300, 200));

        let normalized = normalize_image(&source).unwrap();

        assert_eq!((normalized.width, normalized.height), (300, 200));
        let thumbnail = image::load_from_memory(&normalized.thumbnail).unwrap();
        assert_eq!((thumbnail.width(), thumbnail.height()), (300, 200));
    }

    #[test]
    fn exif_orientation_is_applied_and_every_metadata_is_dropped() {
        let source = with_exif(
            &jpeg_of(&gradient(40, 20)),
            vec![
                field(exif::Tag::Orientation, exif::Value::Short(vec![6])),
                field(
                    exif::Tag::DateTimeOriginal,
                    exif::Value::Ascii(vec![b"2026:11:05 14:03:11".to_vec()]),
                ),
                field(
                    exif::Tag::GPSLatitudeRef,
                    exif::Value::Ascii(vec![b"N".to_vec()]),
                ),
            ],
        );
        assert!(exif_of(&source).is_some());

        let normalized = normalize_image(&source).unwrap();

        assert_eq!((normalized.width, normalized.height), (20, 40));
        assert!(exif_of(&normalized.bytes).is_none());
        assert!(exif_of(&normalized.thumbnail).is_none());
        assert!(!normalized.bytes.windows(4).any(|window| window == b"Exif"));
    }

    #[test]
    fn the_original_capture_time_is_read_before_metadata_is_dropped() {
        let source = with_exif(
            &jpeg_of(&gradient(40, 20)),
            vec![field(
                exif::Tag::DateTimeOriginal,
                exif::Value::Ascii(vec![b"2026:11:05 14:03:11".to_vec()]),
            )],
        );

        let normalized = normalize_image(&source).unwrap();

        assert_eq!(
            normalized.taken_at.map(|moment| moment.to_rfc3339()),
            Some("2026-11-05T14:03:11+00:00".to_owned())
        );
    }

    #[test]
    fn an_image_without_exif_has_no_capture_time() {
        let normalized = normalize_image(&jpeg_of(&gradient(40, 20))).unwrap();

        assert_eq!(normalized.taken_at, None);
    }

    #[test]
    fn a_transparent_png_is_flattened_on_white() {
        let mut source = RgbaImage::from_pixel(10, 10, Rgba([255, 0, 0, 0]));
        source.put_pixel(0, 0, Rgba([0, 0, 255, 255]));

        let normalized = normalize_image(&png_of(&source)).unwrap();

        let stored = image::load_from_memory(&normalized.bytes)
            .unwrap()
            .to_rgb8();
        let corner = stored.get_pixel(9, 9);
        assert!(corner.0.iter().all(|channel| *channel > 245), "{corner:?}");
        let opaque = stored.get_pixel(0, 0);
        assert!(opaque[2] > 200 && opaque[0] < 60, "{opaque:?}");
    }

    #[test]
    fn the_stored_image_is_a_jpeg_and_the_hash_covers_its_bytes() {
        let normalized = normalize_image(&jpeg_of(&gradient(64, 64))).unwrap();

        assert_eq!(
            image::guess_format(&normalized.bytes).unwrap(),
            ImageFormat::Jpeg
        );
        assert_eq!(normalized.content_hash, hex_digest(&normalized.bytes));
        assert_eq!(normalized.content_hash.len(), 64);
    }

    #[test]
    fn the_same_image_always_hashes_the_same() {
        let image = jpeg_of(&gradient(64, 64));

        let first = normalize_image(&image).unwrap();
        let again = normalize_image(&image).unwrap();
        let other = normalize_image(&jpeg_of(&gradient(65, 64))).unwrap();

        assert_eq!(first.content_hash, again.content_hash);
        assert_ne!(first.content_hash, other.content_hash);
    }

    #[test]
    fn a_webp_image_is_accepted() {
        let mut out = Vec::new();
        image::codecs::webp::WebPEncoder::new_lossless(&mut out)
            .write_image(
                gradient(32, 16).as_raw(),
                32,
                16,
                image::ExtendedColorType::Rgb8,
            )
            .unwrap();

        let normalized = normalize_image(&out).unwrap();

        assert_eq!((normalized.width, normalized.height), (32, 16));
    }

    #[test]
    fn a_pdf_is_refused_on_its_content() {
        assert_eq!(
            normalize_image(b"%PDF-1.7\n1 0 obj\n<<>>\nendobj\n"),
            Err(PhotoLibraryError::UnsupportedImage)
        );
    }

    #[test]
    fn an_empty_file_is_refused() {
        assert_eq!(normalize_image(&[]), Err(PhotoLibraryError::EmptyImage));
    }

    #[test]
    fn a_truncated_image_is_refused() {
        let source = jpeg_of(&gradient(200, 200));

        assert_eq!(
            normalize_image(&source[..source.len() / 3]),
            Err(PhotoLibraryError::CorruptImage)
        );
    }

    #[test]
    fn a_truncated_png_is_refused() {
        let source = png_of(&RgbaImage::from_pixel(100, 100, Rgba([10, 200, 30, 255])));

        assert_eq!(
            normalize_image(&source[..source.len() / 2]),
            Err(PhotoLibraryError::CorruptImage)
        );
    }

    #[test]
    fn a_jpeg_with_a_trailer_after_the_end_marker_is_accepted() {
        let mut source = jpeg_of(&gradient(50, 50));
        source.extend_from_slice(b"trailer written by a camera");

        assert!(normalize_image(&source).is_ok());
    }

    #[test]
    fn garbage_behind_a_jpeg_signature_is_refused() {
        let mut source = vec![0xFF, 0xD8, 0xFF, 0xE0];
        source.extend_from_slice(&[7; 64]);

        assert_eq!(
            normalize_image(&source),
            Err(PhotoLibraryError::CorruptImage)
        );
    }

    #[test]
    fn a_gif_is_refused_even_though_the_decoder_could_read_it() {
        assert_eq!(
            normalize_image(b"GIF89a\x01\x00\x01\x00\x00\x00\x00;"),
            Err(PhotoLibraryError::UnsupportedImage)
        );
    }
}
