use ciborium::de::from_reader;
use image::codecs::jpeg::JpegEncoder;
use image::codecs::png::{CompressionType, FilterType as PngFilterType, PngEncoder};
use image::imageops::FilterType as ResizeFilter;
use image::{ColorType, DynamicImage, ExtendedColorType, ImageEncoder, ImageFormat};
use serde::Deserialize;
use wasm_minimal_protocol::*;

initiate_protocol!();

#[derive(Debug, Clone, Deserialize)]
#[serde(default, rename_all = "kebab-case")]
struct OptimiseOptions {
    quality: u8,
    max_width_px: Option<u32>,
    lossy_png: bool,
    strip_metadata: bool,
}

impl Default for OptimiseOptions {
    fn default() -> Self {
        return Self {
            quality: 82,
            max_width_px: None,
            lossy_png: false,
            strip_metadata: true,
        };
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum SourceKind {
    Jpeg,
    Png,
    Svg,
    Pdf,
    Other,
}

#[wasm_func]
pub fn optimise(input: &[u8], options: &[u8]) -> Result<Vec<u8>, String> {
    if input.is_empty() {
        return Err(String::from("input image bytes cannot be empty"));
    }

    let opts = parse_options(options)?;

    return match sniff_source_kind(input) {
        SourceKind::Svg | SourceKind::Pdf => Ok(input.to_vec()),
        SourceKind::Jpeg => optimise_jpeg(input, &opts),
        SourceKind::Png => optimise_png(input, &opts),
        SourceKind::Other => Ok(input.to_vec()),
    };
}

fn parse_options(raw: &[u8]) -> Result<OptimiseOptions, String> {
    if raw.is_empty() {
        return Ok(OptimiseOptions::default());
    }

    let mut options: OptimiseOptions =
        from_reader(raw).map_err(|err| format!("failed to decode options as CBOR: {err}"))?;

    options.quality = options.quality.clamp(1, 100);

    return Ok(options);
}

fn sniff_source_kind(input: &[u8]) -> SourceKind {
    const PNG_MAGIC: [u8; 8] = [0x89, b'P', b'N', b'G', 0x0D, 0x0A, 0x1A, 0x0A];

    if input.starts_with(&[0xFF, 0xD8, 0xFF]) {
        return SourceKind::Jpeg;
    }

    if input.starts_with(&PNG_MAGIC) {
        return SourceKind::Png;
    }

    if input.starts_with(b"%PDF-") {
        return SourceKind::Pdf;
    }

    if looks_like_svg(input) {
        return SourceKind::Svg;
    }

    return match image::guess_format(input) {
        Ok(ImageFormat::Jpeg) => SourceKind::Jpeg,
        Ok(ImageFormat::Png) => SourceKind::Png,
        _ => SourceKind::Other,
    };
}

fn looks_like_svg(input: &[u8]) -> bool {
    let preview = &input[..input.len().min(1024)];
    let trimmed = trim_ascii_start(preview);

    if trimmed.starts_with(b"<svg") {
        return true;
    }

    if trimmed.starts_with(b"<?xml") && preview.windows(4).any(|window| window == b"<svg") {
        return true;
    }

    return false;
}

fn trim_ascii_start(input: &[u8]) -> &[u8] {
    let mut idx = 0;

    while idx < input.len() && input[idx].is_ascii_whitespace() {
        idx += 1;
    }

    return &input[idx..];
}

fn optimise_jpeg(input: &[u8], opts: &OptimiseOptions) -> Result<Vec<u8>, String> {
    let decoded = image::load_from_memory_with_format(input, ImageFormat::Jpeg)
        .map_err(|err| format!("failed to decode JPEG: {err}"))?;

    let (decoded, resized) = maybe_resize(decoded, opts.max_width_px);

    if !opts.strip_metadata && !resized && opts.quality >= 100 {
        return Ok(input.to_vec());
    }

    return encode_jpeg(&decoded, opts.quality);
}

fn optimise_png(input: &[u8], opts: &OptimiseOptions) -> Result<Vec<u8>, String> {
    let decoded = image::load_from_memory_with_format(input, ImageFormat::Png)
        .map_err(|err| format!("failed to decode PNG: {err}"))?;

    let (decoded, resized) = maybe_resize(decoded, opts.max_width_px);

    if opts.lossy_png && !decoded.color().has_alpha() {
        let encoded = encode_jpeg(&decoded, opts.quality)?;

        if !resized && encoded.len() >= input.len() {
            return Ok(input.to_vec());
        }

        return Ok(encoded);
    }

    if !opts.strip_metadata && !resized {
        return Ok(input.to_vec());
    }

    let encoded = encode_png(&decoded)?;

    if !resized && encoded.len() >= input.len() {
        return Ok(input.to_vec());
    }

    return Ok(encoded);
}

fn maybe_resize(image: DynamicImage, max_width_px: Option<u32>) -> (DynamicImage, bool) {
    let Some(max_width_px) = max_width_px else {
        return (image, false);
    };

    if max_width_px == 0 || image.width() <= max_width_px {
        return (image, false);
    }

    let new_height = ((image.height() as f64) * (max_width_px as f64) / (image.width() as f64))
        .round()
        .max(1.0) as u32;

    let resized = image.resize_exact(max_width_px, new_height, ResizeFilter::Lanczos3);

    return (resized, true);
}

fn encode_jpeg(image: &DynamicImage, quality: u8) -> Result<Vec<u8>, String> {
    let mut out = Vec::new();
    let mut encoder = JpegEncoder::new_with_quality(&mut out, quality.clamp(1, 100));

    encoder
        .encode_image(image)
        .map_err(|err| format!("failed to encode JPEG: {err}"))?;

    return Ok(out);
}

fn encode_png(image: &DynamicImage) -> Result<Vec<u8>, String> {
    let mut out = Vec::new();
    let encoder =
        PngEncoder::new_with_quality(&mut out, CompressionType::Fast, PngFilterType::Adaptive);

    match image.color() {
        ColorType::L8 => {
            let luma = image.to_luma8();
            encoder
                .write_image(
                    luma.as_raw(),
                    luma.width(),
                    luma.height(),
                    ExtendedColorType::L8,
                )
                .map_err(|err| format!("failed to encode PNG: {err}"))?;
        }
        ColorType::La8 => {
            let luma_alpha = image.to_luma_alpha8();
            encoder
                .write_image(
                    luma_alpha.as_raw(),
                    luma_alpha.width(),
                    luma_alpha.height(),
                    ExtendedColorType::La8,
                )
                .map_err(|err| format!("failed to encode PNG: {err}"))?;
        }
        ColorType::Rgb8 => {
            let rgb = image.to_rgb8();
            encoder
                .write_image(
                    rgb.as_raw(),
                    rgb.width(),
                    rgb.height(),
                    ExtendedColorType::Rgb8,
                )
                .map_err(|err| format!("failed to encode PNG: {err}"))?;
        }
        ColorType::Rgba8 => {
            let rgba = image.to_rgba8();
            encoder
                .write_image(
                    rgba.as_raw(),
                    rgba.width(),
                    rgba.height(),
                    ExtendedColorType::Rgba8,
                )
                .map_err(|err| format!("failed to encode PNG: {err}"))?;
        }
        _ if image.color().has_alpha() => {
            let rgba = image.to_rgba8();
            encoder
                .write_image(
                    rgba.as_raw(),
                    rgba.width(),
                    rgba.height(),
                    ExtendedColorType::Rgba8,
                )
                .map_err(|err| format!("failed to encode PNG: {err}"))?;
        }
        _ => {
            let rgb = image.to_rgb8();
            encoder
                .write_image(
                    rgb.as_raw(),
                    rgb.width(),
                    rgb.height(),
                    ExtendedColorType::Rgb8,
                )
                .map_err(|err| format!("failed to encode PNG: {err}"))?;
        }
    }

    return Ok(out);
}

#[cfg(test)]
mod optimise_tests {
    use super::*;
    use ciborium::ser::into_writer;
    use image::codecs::png::PngDecoder;
    use image::{ImageDecoder, Rgb, RgbImage};
    use serde::Serialize;
    use std::fs;
    use std::io::Cursor;

    #[derive(Serialize)]
    #[serde(rename_all = "kebab-case")]
    struct TestOptions {
        max_width_px: Option<u32>,
    }

    fn encode_options(max_width_px: Option<u32>) -> Vec<u8> {
        let mut out = Vec::new();

        into_writer(&TestOptions { max_width_px }, &mut out)
            .expect("failed to encode test options");

        return out;
    }

    fn fixture_bytes(path: &str) -> Vec<u8> {
        return fs::read(format!("{}/{}", env!("CARGO_MANIFEST_DIR"), path))
            .expect("failed to read fixture");
    }

    fn synthetic_png_bytes() -> Vec<u8> {
        let image = RgbImage::from_fn(4, 2, |x, y| {
            return Rgb([(x * 40) as u8, (y * 100) as u8, ((x + y) * 30) as u8]);
        });
        let mut out = Vec::new();

        PngEncoder::new_with_quality(&mut out, CompressionType::Best, PngFilterType::Adaptive)
            .write_image(
                image.as_raw(),
                image.width(),
                image.height(),
                ExtendedColorType::Rgb8,
            )
            .expect("failed to encode synthetic PNG");

        return out;
    }

    #[test]
    fn optimise_png_keeps_opaque_pngs_non_alpha_and_non_larger() {
        let input = fixture_bytes("examples/assets/flower.png");
        let output = optimise(&input, &[]).expect("optimise should succeed");
        let decoder = PngDecoder::new(Cursor::new(&output)).expect("output should be a PNG");

        assert!(!decoder.color_type().has_alpha());
        assert!(output.len() <= input.len());
    }

    #[test]
    fn optimise_png_resizes_when_max_width_is_set() {
        let input = synthetic_png_bytes();
        let output = optimise(&input, &encode_options(Some(2))).expect("optimise should succeed");
        let decoder = PngDecoder::new(Cursor::new(&output)).expect("output should be a PNG");

        assert_eq!(decoder.dimensions(), (2, 1));
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ciborium::ser::into_writer;
    use image::{ImageBuffer, Rgb, Rgba};
    use serde::Serialize;
    use std::fs;

    #[derive(Serialize)]
    #[serde(rename_all = "kebab-case")]
    struct TestOptions {
        quality: u8,
        max_width_px: Option<u32>,
        lossy_png: bool,
        strip_metadata: bool,
    }

    impl Default for TestOptions {
        fn default() -> Self {
            return Self {
                quality: 82,
                max_width_px: None,
                lossy_png: false,
                strip_metadata: true,
            };
        }
    }

    fn encode_options(options: TestOptions) -> Vec<u8> {
        let mut out = Vec::new();

        into_writer(&options, &mut out).expect("test options should encode as CBOR");

        return out;
    }

    fn make_rgb_image(width: u32, height: u32) -> DynamicImage {
        let image = ImageBuffer::from_fn(width, height, |x, y| {
            return Rgb([
                ((x * 37 + y * 13) % 255) as u8,
                ((x * 17 + y * 29) % 255) as u8,
                ((x * 11 + y * 23) % 255) as u8,
            ]);
        });

        return DynamicImage::ImageRgb8(image);
    }

    fn make_rgba_image(width: u32, height: u32) -> DynamicImage {
        let image = ImageBuffer::from_fn(width, height, |x, y| {
            let alpha = if (x + y) % 2 == 0 { 255 } else { 0 };

            return Rgba([
                ((x * 41 + y * 19) % 255) as u8,
                ((x * 23 + y * 31) % 255) as u8,
                ((x * 29 + y * 7) % 255) as u8,
                alpha,
            ]);
        });

        return DynamicImage::ImageRgba8(image);
    }

    fn encode_rgb_png(image: &DynamicImage) -> Result<Vec<u8>, String> {
        let rgb = image.to_rgb8();
        let mut out = Vec::new();

        PngEncoder::new_with_quality(&mut out, CompressionType::Best, PngFilterType::Adaptive)
            .write_image(
                rgb.as_raw(),
                rgb.width(),
                rgb.height(),
                ExtendedColorType::Rgb8,
            )
            .map_err(|err| format!("failed to encode RGB PNG for test: {err}"))?;

        return Ok(out);
    }

    fn fixture_bytes(path: &str) -> Vec<u8> {
        return fs::read(format!("{}/{}", env!("CARGO_MANIFEST_DIR"), path))
            .expect("failed to read fixture");
    }

    #[test]
    fn parse_options_returns_defaults_for_empty_input() {
        let options = parse_options(&[]).expect("empty options should fall back to defaults");

        assert_eq!(options.quality, 82);
        assert_eq!(options.max_width_px, None);
        assert!(!options.lossy_png);
        assert!(options.strip_metadata);
    }

    #[test]
    fn parse_options_clamps_quality_bounds() {
        let lower = parse_options(&encode_options(TestOptions {
            quality: 0,
            ..TestOptions::default()
        }))
        .expect("quality lower bound should parse");
        let upper = parse_options(&encode_options(TestOptions {
            quality: 255,
            ..TestOptions::default()
        }))
        .expect("quality upper bound should parse");

        assert_eq!(lower.quality, 1);
        assert_eq!(upper.quality, 100);
    }

    #[test]
    fn sniff_source_kind_detects_supported_formats() {
        let jpeg = encode_jpeg(&make_rgb_image(3, 2), 75).expect("JPEG test image should encode");
        let png = encode_rgb_png(&make_rgb_image(3, 2)).expect("PNG test image should encode");

        assert_eq!(sniff_source_kind(&jpeg), SourceKind::Jpeg);
        assert_eq!(sniff_source_kind(&png), SourceKind::Png);
        assert_eq!(sniff_source_kind(b"%PDF-1.7\n"), SourceKind::Pdf);
        assert_eq!(
            sniff_source_kind(b"  <?xml version=\"1.0\"?><svg viewBox=\"0 0 1 1\"></svg>"),
            SourceKind::Svg,
        );
        assert_eq!(sniff_source_kind(b"plain text"), SourceKind::Other);
    }

    #[test]
    fn optimise_rejects_empty_input() {
        let err = optimise(&[], &[]).expect_err("empty input should be rejected");

        assert!(err.contains("cannot be empty"));
    }

    #[test]
    fn optimise_passes_through_non_raster_inputs() {
        let svg = b"<svg viewBox=\"0 0 1 1\"></svg>";
        let pdf = b"%PDF-1.7\n1 0 obj\n<<>>\nendobj\n";
        let other = b"plain text";

        assert_eq!(optimise(svg, &[]).expect("SVG should pass through"), svg);
        assert_eq!(optimise(pdf, &[]).expect("PDF should pass through"), pdf);
        assert_eq!(
            optimise(other, &[]).expect("other bytes should pass through"),
            other
        );
    }

    #[test]
    fn optimise_jpeg_returns_original_when_no_change_is_requested() {
        let input = encode_jpeg(&make_rgb_image(6, 4), 70).expect("JPEG test image should encode");
        let options = encode_options(TestOptions {
            quality: 100,
            strip_metadata: false,
            ..TestOptions::default()
        });

        let output = optimise(&input, &options).expect("JPEG should optimise successfully");

        assert_eq!(output, input);
    }

    #[test]
    fn optimise_png_returns_original_when_metadata_is_preserved() {
        let input = encode_png(&make_rgba_image(6, 4)).expect("PNG test image should encode");
        let options = encode_options(TestOptions {
            strip_metadata: false,
            ..TestOptions::default()
        });

        let output = optimise(&input, &options).expect("PNG should optimise successfully");

        assert_eq!(output, input);
    }

    #[test]
    fn optimise_converts_opaque_png_to_jpeg_when_lossy_png_is_enabled() {
        let input = fixture_bytes("examples/assets/flower.png");
        let options = encode_options(TestOptions {
            lossy_png: true,
            quality: 68,
            ..TestOptions::default()
        });

        let input_decoded = image::load_from_memory_with_format(&input, ImageFormat::Png)
            .expect("input PNG should decode");
        let output = optimise(&input, &options).expect("opaque PNG should convert to JPEG");
        let decoded = image::load_from_memory(&output).expect("output image should decode");

        assert_eq!(sniff_source_kind(&output), SourceKind::Jpeg);
        assert_eq!(decoded.width(), input_decoded.width());
        assert_eq!(decoded.height(), input_decoded.height());
    }

    #[test]
    fn optimise_keeps_alpha_png_lossless_when_lossy_png_is_enabled() {
        let input = encode_png(&make_rgba_image(5, 3)).expect("PNG test image should encode");
        let options = encode_options(TestOptions {
            lossy_png: true,
            ..TestOptions::default()
        });

        let output = optimise(&input, &options).expect("transparent PNG should stay as PNG");
        let decoded = image::load_from_memory_with_format(&output, ImageFormat::Png)
            .expect("output PNG should decode");

        assert_eq!(sniff_source_kind(&output), SourceKind::Png);
        assert!(decoded.color().has_alpha());
        assert_eq!(decoded.width(), 5);
        assert_eq!(decoded.height(), 3);
    }

    #[test]
    fn optimise_resizes_raster_images_to_the_requested_width() {
        let input = encode_jpeg(&make_rgb_image(8, 4), 80).expect("JPEG test image should encode");
        let options = encode_options(TestOptions {
            quality: 100,
            max_width_px: Some(4),
            strip_metadata: false,
            ..TestOptions::default()
        });

        let output = optimise(&input, &options).expect("JPEG resize should succeed");
        let decoded = image::load_from_memory(&output).expect("resized JPEG should decode");

        assert_eq!(decoded.width(), 4);
        assert_eq!(decoded.height(), 2);
    }
}
