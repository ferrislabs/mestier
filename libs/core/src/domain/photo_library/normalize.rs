use std::io::Cursor;

use chrono::{DateTime, FixedOffset, NaiveDate, Utc};
use image::{
    DynamicImage, ImageDecoder, ImageError, ImageFormat, ImageReader, Limits, RgbImage,
    codecs::jpeg::JpegEncoder, imageops::FilterType, metadata::Orientation,
};
use sha2::{Digest, Sha256};

use crate::domain::photo_library::error::{PhotoLibraryError, PhotoLibraryOutcome};

pub const MAX_SIDE: u32 = 2400;
pub const THUMBNAIL_SIDE: u32 = 400;
pub const STORED_MIME_TYPE: &str = "image/jpeg";

const IMAGE_QUALITY: u8 = 85;
const THUMBNAIL_QUALITY: u8 = 75;
const RESIZE_BYTES_PER_PIXEL: u64 = 16;
const FLATTENED_BYTES_PER_PIXEL: u64 = 3;
const ALPHA_CONVERSION_BYTES_PER_PIXEL: u64 = 4;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PixelBudget {
    pub max_pixels: u64,
    pub max_working_bytes: u64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DecodeLimits {
    pub max_side: u32,
    pub max_alloc: u64,
    pub pixel_budget: Option<PixelBudget>,
}

pub const LIBRARY_DECODE_LIMITS: DecodeLimits = DecodeLimits {
    max_side: 20_000,
    max_alloc: 768 * 1024 * 1024,
    pixel_budget: None,
};

pub const SURVEY_DECODE_LIMITS: DecodeLimits = DecodeLimits {
    max_side: 12_000,
    max_alloc: 640 * 1024 * 1024,
    pixel_budget: Some(PixelBudget {
        max_pixels: 50_000_000,
        max_working_bytes: 640 * 1024 * 1024,
    }),
};

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
    normalize_image_with(input, LIBRARY_DECODE_LIMITS)
}

pub fn normalize_image_with(
    input: &[u8],
    limits: DecodeLimits,
) -> PhotoLibraryOutcome<NormalizedImage> {
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

    if let Some(budget) = limits.pixel_budget {
        within_budget(input, format, limits, budget)?;
    }

    let metadata = read_metadata(input);

    let mut reader = ImageReader::with_format(Cursor::new(input), format);
    reader.limits(decoder_limits(limits));
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

fn decoder_limits(limits: DecodeLimits) -> Limits {
    let mut decoder = Limits::default();
    decoder.max_image_width = Some(limits.max_side);
    decoder.max_image_height = Some(limits.max_side);
    decoder.max_alloc = Some(limits.max_alloc);
    decoder
}

fn within_budget(
    input: &[u8],
    format: ImageFormat,
    limits: DecodeLimits,
    budget: PixelBudget,
) -> PhotoLibraryOutcome<()> {
    let mut reader = ImageReader::with_format(Cursor::new(input), format);
    reader.limits(decoder_limits(limits));
    let decoder = reader.into_decoder().map_err(|error| match error {
        ImageError::Limits(_) => PhotoLibraryError::ImageTooLarge,
        _ => PhotoLibraryError::CorruptImage,
    })?;
    let (width, height) = decoder.dimensions();
    let color = decoder.color_type();
    let fits = width.max(height) <= limits.max_side
        && u64::from(width) * u64::from(height) <= budget.max_pixels
        && working_bytes(
            width,
            height,
            u64::from(color.bytes_per_pixel()),
            color.has_alpha(),
        ) <= budget.max_working_bytes;
    if fits {
        Ok(())
    } else {
        Err(PhotoLibraryError::ImageTooLarge)
    }
}

pub fn working_bytes(width: u32, height: u32, bytes_per_pixel: u64, has_alpha: bool) -> u64 {
    let pixels = u64::from(width) * u64::from(height);
    let decoded = pixels.saturating_mul(bytes_per_pixel);
    let oriented = decoded.saturating_mul(2);
    let conversion = if has_alpha {
        ALPHA_CONVERSION_BYTES_PER_PIXEL + FLATTENED_BYTES_PER_PIXEL
    } else {
        FLATTENED_BYTES_PER_PIXEL
    };
    let flattened = decoded.saturating_add(pixels.saturating_mul(conversion));
    let intermediate = if width.max(height) > MAX_SIDE {
        u64::from(width.min(height)) * u64::from(MAX_SIDE) * RESIZE_BYTES_PER_PIXEL
    } else {
        0
    };
    let resized = pixels
        .saturating_mul(FLATTENED_BYTES_PER_PIXEL)
        .saturating_add(intermediate);
    oriented.max(flattened).max(resized)
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

    fn crc32(bytes: &[u8]) -> u32 {
        let mut crc = 0xFFFF_FFFFu32;
        for byte in bytes {
            crc ^= u32::from(*byte);
            for _ in 0..8 {
                crc = if crc & 1 == 1 {
                    (crc >> 1) ^ 0xEDB8_8320
                } else {
                    crc >> 1
                };
            }
        }
        !crc
    }

    fn png_declaring(width: u32, height: u32) -> Vec<u8> {
        let mut png = png_of(&RgbaImage::from_pixel(1, 1, Rgba([1, 2, 3, 255])));
        png[16..20].copy_from_slice(&width.to_be_bytes());
        png[20..24].copy_from_slice(&height.to_be_bytes());
        let crc = crc32(&png[12..29]);
        png[29..33].copy_from_slice(&crc.to_be_bytes());
        png
    }

    #[test]
    fn the_library_keeps_its_decode_limits() {
        assert_eq!(
            LIBRARY_DECODE_LIMITS,
            DecodeLimits {
                max_side: 20_000,
                max_alloc: 768 * 1024 * 1024,
                pixel_budget: None,
            }
        );
        assert_eq!(
            normalize_image(&png_declaring(30_000, 30_000)),
            Err(PhotoLibraryError::CorruptImage)
        );
    }

    #[test]
    fn a_survey_photo_declaring_thirty_thousand_pixels_a_side_is_refused_from_its_header() {
        assert_eq!(
            normalize_image_with(&png_declaring(30_000, 30_000), SURVEY_DECODE_LIMITS),
            Err(PhotoLibraryError::ImageTooLarge)
        );
    }

    #[test]
    fn a_survey_photo_over_fifty_megapixels_is_refused_before_decoding() {
        assert_eq!(
            normalize_image_with(&png_declaring(11_000, 11_000), SURVEY_DECODE_LIMITS),
            Err(PhotoLibraryError::ImageTooLarge)
        );
        assert_eq!(
            normalize_image_with(&png_declaring(12_001, 100), SURVEY_DECODE_LIMITS),
            Err(PhotoLibraryError::ImageTooLarge)
        );
    }

    #[test]
    fn a_survey_photo_within_the_budget_is_decoded() {
        let normalized =
            normalize_image_with(&jpeg_of(&gradient(3000, 1000)), SURVEY_DECODE_LIMITS).unwrap();

        assert_eq!((normalized.width, normalized.height), (2400, 800));
        assert_eq!(
            normalize_image_with(&png_declaring(4000, 3000), SURVEY_DECODE_LIMITS),
            Err(PhotoLibraryError::CorruptImage)
        );
    }

    #[test]
    fn the_survey_budget_counts_the_resize_buffer_and_fits_a_fifty_megapixel_photo() {
        let Some(budget) = SURVEY_DECODE_LIMITS.pixel_budget else {
            panic!("the survey limits carry a pixel budget");
        };
        assert_eq!(budget.max_pixels, 50_000_000);
        assert_eq!(SURVEY_DECODE_LIMITS.max_side, 12_000);
        assert!(working_bytes(8165, 6124, 3, false) <= budget.max_working_bytes);
        assert!(working_bytes(7071, 7071, 4, true) <= budget.max_working_bytes);
        assert!(working_bytes(7071, 7071, 8, true) > budget.max_working_bytes);
        assert_eq!(
            working_bytes(10_000, 2_000, 3, false),
            10_000 * 2_000 * 3 + 2_000 * 2_400 * 16
        );
        assert_eq!(working_bytes(2_000, 1_000, 3, false), 2_000 * 1_000 * 6);
    }

    #[test]
    fn a_gif_is_refused_even_though_the_decoder_could_read_it() {
        assert_eq!(
            normalize_image(b"GIF89a\x01\x00\x01\x00\x00\x00\x00;"),
            Err(PhotoLibraryError::UnsupportedImage)
        );
    }
}
