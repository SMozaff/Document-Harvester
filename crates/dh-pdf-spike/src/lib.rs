//! Isolated Phase 0 evidence probe for the candidate PDF backend.
//!
//! This crate is deliberately **not** a production dependency. No product crate
//! (`dh-core`, `dh-writer`, `dh-part1`, `dh-part2`, `dh-part3`, `dh-ui`,
//! `dh-app`) may depend on it. It exists only to record measured behaviour of
//! the PDF backend candidates listed in `docs/MVP_PHASE0_FINDINGS.md`, as
//! required for Stage 2 task 2.1 by manifest v5.1 Appendix C.2 and
//! `docs/DEPENDENCIES.md`.
//!
//! # What this crate does and does not prove
//!
//! It reports what the backend emits for a given document. That is an
//! observation, not ground truth. Correctness of character/span geometry,
//! reading order, font metadata, page boxes and rendering can only be judged
//! against independently annotated fixtures, which do not exist yet (owner
//! decision A). Until those exist, nothing in this crate supports an accuracy
//! or acceptance claim.
//!
//! The one behaviour that is checked without fixtures is failure handling:
//! malformed input must return an error rather than panic or report an empty
//! document.

use std::path::Path;

use pdf_oxide::api::Pdf;
use pdf_oxide::rendering::{ImageFormat, RenderOptions};

/// Rectangle as reported by the backend, in PDF page user space (points).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct BackendBox {
    /// Left edge.
    pub x: f32,
    /// Top edge.
    pub y: f32,
    /// Width.
    pub width: f32,
    /// Height.
    pub height: f32,
}

impl From<pdf_oxide::geometry::Rect> for BackendBox {
    fn from(rect: pdf_oxide::geometry::Rect) -> Self {
        Self {
            x: rect.x,
            y: rect.y,
            width: rect.width,
            height: rect.height,
        }
    }
}

/// Options controlling how much of a document the probe reads.
#[derive(Debug, Clone, Copy)]
pub struct ProbeOptions {
    /// Render every page at this DPI. `None` skips rendering entirely.
    pub render_dpi: Option<u32>,
    /// Stop after this many pages. `None` reads every page.
    pub max_pages: Option<usize>,
    /// How many leading characters and spans to sample per page.
    pub sample: usize,
}

impl Default for ProbeOptions {
    fn default() -> Self {
        Self {
            render_dpi: None,
            max_pages: None,
            sample: 8,
        }
    }
}

/// A single character as the backend reports it.
#[derive(Debug, Clone)]
pub struct CharObservation {
    /// The extracted character.
    pub ch: char,
    /// Backend bounding box.
    pub bbox: BackendBox,
    /// Font name the backend associates with the glyph.
    pub font_name: String,
    /// Font size in points.
    pub font_size: f32,
    /// Baseline origin X.
    pub origin_x: f32,
    /// Baseline origin Y.
    pub origin_y: f32,
    /// Text rotation in degrees.
    pub rotation_degrees: f32,
    /// Advance to the next glyph, excluding `Tc`/`Tw`/`TJ` adjustments.
    pub advance_width: f32,
}

/// A single text span (one `Tj`/`TJ` string) as the backend reports it.
#[derive(Debug, Clone)]
pub struct SpanObservation {
    /// The span text.
    pub text: String,
    /// Backend bounding box.
    pub bbox: BackendBox,
    /// Font name the backend associates with the span.
    pub font_name: String,
    /// Font size in points.
    pub font_size: f32,
    /// Raw extraction sequence number.
    pub sequence: usize,
}

/// Result of rendering one page.
#[derive(Debug, Clone, Copy)]
pub struct RenderObservation {
    /// Pixel width.
    pub width: u32,
    /// Pixel height.
    pub height: u32,
    /// Format requested from the backend.
    pub format: &'static str,
    /// Byte length of the returned buffer.
    pub byte_len: usize,
}

/// Everything the probe observed for one page.
#[derive(Debug, Clone)]
pub struct PageObservation {
    /// Zero-based page index.
    pub index: usize,
    /// Page rotation in degrees as reported by the backend.
    pub rotation_degrees: i32,
    /// `/MediaBox` as `[llx, lly, urx, ury]`.
    pub media_box: [f32; 4],
    /// `/CropBox` as `[llx, lly, urx, ury]`, or `None` when unset.
    pub crop_box: Option<[f32; 4]>,
    /// Number of characters the backend extracted.
    pub char_count: usize,
    /// Number of spans the backend extracted.
    pub span_count: usize,
    /// Leading characters, in the backend's raw order.
    pub first_chars: Vec<CharObservation>,
    /// Leading spans, in the backend's raw order.
    pub first_spans: Vec<SpanObservation>,
    /// Render result, when rendering was requested.
    pub render: Option<RenderObservation>,
}

/// Everything the probe observed for one document.
#[derive(Debug, Clone)]
pub struct DocumentObservation {
    /// Total page count reported by the backend.
    pub page_count: usize,
    /// Observations for the pages that were actually read.
    pub pages: Vec<PageObservation>,
}

impl DocumentObservation {
    /// Render this observation as a plain-text report.
    ///
    /// The report states observed values only. It is not a correctness claim.
    pub fn to_report(&self) -> String {
        let mut out = String::new();
        out.push_str("backend: pdf_oxide 0.3.78\n");
        out.push_str("report: observed backend output, not ground truth\n");
        out.push_str(&format!("page_count: {}\n", self.page_count));
        for page in &self.pages {
            out.push_str(&format!(
                "page {}: rotation={} media_box={:?} crop_box={:?} chars={} spans={}\n",
                page.index,
                page.rotation_degrees,
                page.media_box,
                page.crop_box,
                page.char_count,
                page.span_count,
            ));
            if let Some(render) = page.render {
                out.push_str(&format!(
                    "  render: {} {}x{} bytes={}\n",
                    render.format, render.width, render.height, render.byte_len
                ));
            }
            for span in &page.first_spans {
                out.push_str(&format!(
                    "  span[{}] {:?} bbox=({}, {}) {}x{} font={:?} size={}\n",
                    span.sequence,
                    span.text,
                    span.bbox.x,
                    span.bbox.y,
                    span.bbox.width,
                    span.bbox.height,
                    span.font_name,
                    span.font_size,
                ));
            }
            for ch in &page.first_chars {
                out.push_str(&format!(
                    "  char {:?} bbox=({}, {}) {}x{} font={:?} size={} origin=({}, {}) rot={} advance={}\n",
                    ch.ch,
                    ch.bbox.x,
                    ch.bbox.y,
                    ch.bbox.width,
                    ch.bbox.height,
                    ch.font_name,
                    ch.font_size,
                    ch.origin_x,
                    ch.origin_y,
                    ch.rotation_degrees,
                    ch.advance_width,
                ));
            }
        }
        out
    }
}

/// Probe an on-disk PDF and return what the backend reports.
///
/// Returns the backend's error unchanged so callers can record the exact
/// diagnostic. Malformed, encrypted, unsupported and image-only documents are
/// expected to arrive here as `Err`.
pub fn probe_path(
    path: impl AsRef<Path>,
    options: ProbeOptions,
) -> pdf_oxide::Result<DocumentObservation> {
    let mut pdf = Pdf::open(path)?;
    let page_count = pdf.page_count()?;
    let limit = match options.max_pages {
        Some(max) => page_count.min(max),
        None => page_count,
    };

    let mut pages = Vec::with_capacity(limit);
    for index in 0..limit {
        pages.push(probe_page(&mut pdf, index, &options)?);
    }

    Ok(DocumentObservation { page_count, pages })
}

fn probe_page(
    pdf: &mut Pdf,
    index: usize,
    options: &ProbeOptions,
) -> pdf_oxide::Result<PageObservation> {
    let rotation_degrees = pdf.page_rotation(index)?;
    let media_box = pdf.page_media_box(index)?;
    let crop_box = pdf.page_crop_box(index)?;

    let chars = pdf.extract_chars(index)?;
    let spans = pdf.extract_spans(index)?;

    let first_chars = chars
        .iter()
        .take(options.sample)
        .map(|ch| CharObservation {
            ch: ch.char,
            bbox: ch.bbox.into(),
            font_name: ch.font_name.clone(),
            font_size: ch.font_size,
            origin_x: ch.origin_x,
            origin_y: ch.origin_y,
            rotation_degrees: ch.rotation_degrees,
            advance_width: ch.advance_width,
        })
        .collect();

    let first_spans = spans
        .iter()
        .take(options.sample)
        .map(|span| SpanObservation {
            text: span.text.clone(),
            bbox: span.bbox.into(),
            font_name: span.font_name.clone(),
            font_size: span.font_size,
            sequence: span.sequence,
        })
        .collect();

    let render = match options.render_dpi {
        Some(dpi) => {
            // `RenderOptions` has a private field, so struct-update syntax from
            // outside the crate does not compile. The builder below is the
            // supported route.
            let render_options = RenderOptions::with_dpi(dpi).as_raw();
            debug_assert!(matches!(render_options.format, ImageFormat::RawRgba8));
            let image = pdf.render_page(index, Some(&render_options))?;
            Some(RenderObservation {
                width: image.width,
                height: image.height,
                format: "RawRgba8",
                byte_len: image.data.len(),
            })
        }
        None => None,
    };

    Ok(PageObservation {
        index,
        rotation_degrees,
        media_box,
        crop_box,
        char_count: chars.len(),
        span_count: spans.len(),
        first_chars,
        first_spans,
        render,
    })
}

#[cfg(test)]
mod tests {
    use std::panic::{catch_unwind, AssertUnwindSafe};

    use super::*;

    /// Malformed input must fail with an error, never panic or report an empty
    /// document. This is the only probe behaviour that needs no fixture.
    #[test]
    fn malformed_input_is_reported_as_an_error() {
        let dir = std::env::temp_dir().join(format!("dh-pdf-spike-{}", std::process::id()));
        std::fs::create_dir_all(&dir).expect("create temp dir");
        let path = dir.join("not-a-pdf.pdf");
        std::fs::write(&path, b"this is not a PDF file").expect("write malformed input");

        let outcome = catch_unwind(AssertUnwindSafe(|| {
            probe_path(&path, ProbeOptions::default())
        }));

        std::fs::remove_dir_all(&dir).ok();

        let result = outcome.expect("malformed input must not panic");
        assert!(
            result.is_err(),
            "malformed input must be reported as Err, got {:?}",
            result.map(|doc| doc.page_count)
        );
    }

    /// A missing file must fail with an error, never panic.
    #[test]
    fn missing_file_is_reported_as_an_error() {
        let path = std::env::temp_dir().join(format!(
            "dh-pdf-spike-absent-{}-{}.pdf",
            std::process::id(),
            "does-not-exist"
        ));
        let outcome = catch_unwind(AssertUnwindSafe(|| {
            probe_path(&path, ProbeOptions::default())
        }));

        let result = outcome.expect("a missing file must not panic");
        assert!(result.is_err(), "a missing file must be reported as Err");
    }

    /// Probe every `*.pdf` in `DH_PDF_FIXTURES`.
    ///
    /// The assertion is only that each document either probes or returns an
    /// error without panicking. It makes no accuracy claim: expected values
    /// must come from independent annotation (owner decision A), which does
    /// not exist yet.
    #[test]
    #[ignore = "blocked on owner decision A: no permitted, independently annotated fixture PDFs exist yet; set DH_PDF_FIXTURES to a directory of PDFs to run"]
    fn fixtures_probe_or_fail_explicitly_without_panicking() {
        let dir = std::env::var("DH_PDF_FIXTURES")
            .expect("set DH_PDF_FIXTURES to a directory containing fixture PDFs");

        let mut probed = 0usize;
        let mut failed = 0usize;
        for entry in std::fs::read_dir(&dir).expect("read DH_PDF_FIXTURES") {
            let path = entry.expect("read dir entry").path();
            if path.extension().and_then(|ext| ext.to_str()) != Some("pdf") {
                continue;
            }
            let outcome = catch_unwind(AssertUnwindSafe(|| {
                probe_path(&path, ProbeOptions::default())
            }));
            let result = outcome.unwrap_or_else(|_| panic!("panicked while probing {path:?}"));
            match result {
                Ok(document) => {
                    probed += 1;
                    assert!(
                        document.page_count > 0,
                        "{path:?} opened but reported zero pages"
                    );
                    assert_eq!(
                        document.pages.len(),
                        document.page_count,
                        "{path:?} page observations do not match the reported page count"
                    );
                }
                Err(error) => {
                    failed += 1;
                    eprintln!("{path:?} failed explicitly: {error}");
                }
            }
        }

        assert!(
            probed + failed > 0,
            "no PDF fixtures found in {dir}; the gate cannot pass without fixtures"
        );
    }
}
