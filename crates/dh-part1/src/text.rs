//! Digital text extraction through the already-adopted bundled PDFium.
//!
//! PDFium character order is recorded as visual/document-defined order; it is
//! never relabelled as logical reading order. Font fields PDFium does not
//! expose are listed in `missing_font_metadata` / `unavailable_fields` rather
//! than invented. Rendered pixels are never used for extraction.

use std::{path::Path, sync::Arc};

use dh_ir::{
    BBox, DhError, FontIdentity, OrderEvidenceSource, PageTextExtraction, PositionedTextRun,
    Result, TextOrder, TextRunOrder,
};
use pdfium_render::prelude::*;

use crate::renderer::{bind_pdfium, bundled_library_path, effective_page_box, pdf_error};

/// PDFium adapter for positioned digital text.
#[derive(Clone)]
pub struct PdfiumTextExtractor {
    pdfium: Arc<Pdfium>,
}

impl PdfiumTextExtractor {
    pub fn load_bundled() -> Result<Self> {
        Self::with_library(&bundled_library_path()?)
    }

    pub fn with_library(library_path: &Path) -> Result<Self> {
        Ok(Self {
            pdfium: bind_pdfium(library_path)?,
        })
    }
}

impl dh_ir::PageTextExtractor for PdfiumTextExtractor {
    fn page_count(&self, pdf_path: &Path) -> Result<u32> {
        let document = self
            .pdfium
            .load_pdf_from_file(pdf_path, None)
            .map_err(pdf_error)?;
        u32::try_from(document.pages().len())
            .map_err(|_| DhError::Parse("PDF has too many pages".to_owned()))
    }

    fn extract_page(&self, pdf_path: &Path, page_number: u32) -> Result<PageTextExtraction> {
        if page_number == 0 {
            return Err(DhError::InvalidInput("page numbers start at 1".to_owned()));
        }
        let document = self
            .pdfium
            .load_pdf_from_file(pdf_path, None)
            .map_err(pdf_error)?;
        let index = i32::try_from(page_number - 1)
            .map_err(|_| DhError::InvalidInput("page number is too large".to_owned()))?;
        let page = document.pages().get(index).map_err(pdf_error)?;
        let page_box = effective_page_box(&page)?;
        let rotation_degrees = page.rotation().map_err(pdf_error)?.as_degrees() as u16;
        let text_page = page.text().map_err(pdf_error)?;

        if text_page.is_empty() {
            return Ok(PageTextExtraction {
                page: page_number,
                page_box,
                rotation_degrees,
                runs: Vec::new(),
                unsupported_reason: Some(
                    "no digital text on page; image-only and blank pages are unsupported"
                        .to_owned(),
                ),
            });
        }

        let mut observed = Vec::new();
        for character in text_page.chars().iter() {
            let bounds = character.tight_bounds().map_err(pdf_error)?;
            let bbox = BBox {
                x0: f64::from(bounds.left().value),
                y0: f64::from(bounds.bottom().value),
                x1: f64::from(bounds.right().value),
                y1: f64::from(bounds.top().value),
            };
            let font_name = character.font_name();
            let font_size_points = f64::from(character.unscaled_font_size().value);
            let weight = character
                .font_weight()
                .map(|weight| format!("{weight:?}"))
                .unwrap_or_default();
            let italic = character.font_is_italic();
            let bold = character.font_is_bold_reenforced()
                || matches!(
                    character.font_weight(),
                    Some(PdfFontWeight::Weight700Bold)
                        | Some(PdfFontWeight::Weight800)
                        | Some(PdfFontWeight::Weight900)
                );
            let angle_degrees = character.angle_degrees().unwrap_or(0.0);
            // pdfium-render's helper returns true when there is NO mapping
            // error (its implementation inverts the raw FPDF flag).
            let has_unicode_error = !character.has_unicode_map_error().unwrap_or(false);
            let text = character
                .unicode_string()
                .unwrap_or_else(|| "\u{FFFD}".to_owned());
            observed.push(ObservedChar {
                index: u32::try_from(character.index()).unwrap_or(u32::MAX),
                text,
                bbox,
                font_name,
                font_size_points,
                weight,
                italic,
                bold,
                angle_degrees,
                has_unicode_error,
            });
        }

        let mut runs = Vec::new();
        let mut current: Option<ObservedChar> = None;
        let mut group: Vec<ObservedChar> = Vec::new();
        for character in observed {
            let is_whitespace = character.text.chars().all(char::is_whitespace);
            let can_join = current
                .as_ref()
                .is_some_and(|first| same_run(first, &character));
            if can_join {
                group.push(character);
                if is_whitespace {
                    if let Some(first) = current.take() {
                        group.insert(0, first);
                        runs.push(make_run(page_number, &group));
                        group.clear();
                    }
                }
                continue;
            }

            if let Some(first) = current.take() {
                group.insert(0, first);
                runs.push(make_run(page_number, &group));
                group.clear();
            }
            if is_whitespace {
                runs.push(make_run(page_number, std::slice::from_ref(&character)));
            } else {
                current = Some(character);
            }
        }
        if let Some(first) = current {
            group.insert(0, first);
            runs.push(make_run(page_number, &group));
        }

        Ok(PageTextExtraction {
            page: page_number,
            page_box,
            rotation_degrees,
            runs,
            unsupported_reason: None,
        })
    }
}

#[derive(Clone)]
struct ObservedChar {
    index: u32,
    text: String,
    bbox: BBox,
    font_name: String,
    font_size_points: f64,
    weight: String,
    italic: bool,
    bold: bool,
    angle_degrees: f32,
    has_unicode_error: bool,
}

fn same_run(first: &ObservedChar, next: &ObservedChar) -> bool {
    let font_size = first.font_size_points.abs().max(f64::MIN_POSITIVE);
    let baseline_difference = (first.bbox.y0 - next.bbox.y0).abs();
    let gap = next.bbox.x0 - first.bbox.x1;
    let is_whitespace = first.text.chars().all(char::is_whitespace);

    !first.has_unicode_error
        && !next.has_unicode_error
        && first.font_name == next.font_name
        && (first.font_size_points - next.font_size_points).abs() <= 0.25
        && first.italic == next.italic
        && first.bold == next.bold
        && (first.angle_degrees - next.angle_degrees).abs() <= 1.0
        && baseline_difference <= font_size.max(2.0)
        && gap <= font_size * 0.5
        && next.index == first.index + 1
        && !is_whitespace
}

fn make_run(page: u32, group: &[ObservedChar]) -> PositionedTextRun {
    let first = group.first().expect("a run has at least one character");
    let mut bbox = first.bbox;
    let mut text = String::new();
    let mut has_unicode_error = false;
    for character in group {
        bbox = BBox {
            x0: bbox.x0.min(character.bbox.x0),
            y0: bbox.y0.min(character.bbox.y0),
            x1: bbox.x1.max(character.bbox.x1),
            y1: bbox.y1.max(character.bbox.y1),
        };
        text.push_str(&character.text);
        has_unicode_error |= character.has_unicode_error;
    }
    let run_id = format!("p{p}-r{index:04}", p = page, index = first.index);
    let mut missing_font_metadata = vec![
        "font-subtype".to_owned(),
        "embedded-flag".to_owned(),
        "program-fingerprint".to_owned(),
        "encoding".to_owned(),
        "cmap-id".to_owned(),
        "tounicode-presence".to_owned(),
        "tounicode-fingerprint".to_owned(),
        "cid-system-info".to_owned(),
        "resource-scope-id".to_owned(),
    ];
    if first.weight.is_empty() {
        missing_font_metadata.push("font-weight".to_owned());
    }
    if has_unicode_error {
        missing_font_metadata.push("unicode-mapping".to_owned());
    }
    let unavailable_fields = missing_font_metadata.clone();
    let font = FontIdentity {
        name_normalized: first.font_name.trim().to_owned(),
        subtype: String::new(),
        embedded: false,
        program_fingerprint: None,
        encoding: None,
        cmap_id: None,
        encoding_cmap_fingerprint: None,
        has_tounicode: false,
        tounicode_fingerprint: None,
        cid_system_info_registry: None,
        cid_system_info_ordering: None,
        cid_system_info_supplement: None,
        resource_scope_id: None,
        unavailable_fields,
    };

    PositionedTextRun {
        run_id: run_id.clone(),
        text,
        bbox,
        font_name: first.font_name.clone(),
        font_size_points: first.font_size_points,
        font_weight: if first.weight.is_empty() {
            None
        } else {
            Some(first.weight.clone())
        },
        italic: first.italic,
        bold: first.bold,
        angle_degrees: first.angle_degrees,
        font,
        order: TextRunOrder {
            run_id,
            order: TextOrder::Visual,
            source: OrderEvidenceSource::ExtractorRunMetadata,
            sequence_index: Some(u64::from(first.index)),
            writing_mode: None,
            rtl_draw_logical: None,
            rotation_degrees: Some(first.angle_degrees),
        },
        missing_font_metadata,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn vendored_library_is_resolved_from_the_repository() {
        let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../../vendor/pdfium")
            .join(if cfg!(windows) {
                "win-x64"
            } else {
                "linux-x64"
            })
            .join(Pdfium::pdfium_platform_library_name());
        assert!(
            path.is_file(),
            "missing vendored PDFium: {}",
            path.display()
        );
        let _extractor = PdfiumTextExtractor::with_library(&path).expect("load PDFium");
    }
}
