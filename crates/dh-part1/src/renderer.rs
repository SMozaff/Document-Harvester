//! Viewer-only page rendering through the bundled PDFium library.
//!
//! This is the single renderer implementation used by the desktop viewer. It
//! never supplies pixels to extraction; digital text extraction must use the
//! PDF's own text data. A document is opened for the duration of one call so
//! the renderer stays `Send + Sync` and the Slint UI thread only ever receives
//! plain pixel data.

use std::{
    ffi::OsString,
    path::{Path, PathBuf},
    sync::{Arc, OnceLock},
};

use dh_ir::{BBox, DhError, Result};
use pdfium_render::prelude::*;

use crate::diagnostics::{load_error_diagnostic, PdfMeasurements, PdfOpenDiagnostics};

/// Geometry and identity of one page, in unrotated PDF user space.
#[derive(Debug, Clone, PartialEq)]
pub struct PageInfo {
    pub page_count: u32,
    /// One-based page number, matching the viewer's navigation controls.
    pub page_number: u32,
    pub width_points: f64,
    pub height_points: f64,
    /// Clockwise quarter-turn rotation normalized to 0, 90, 180 or 270.
    pub rotation_degrees: u16,
    /// Effective box for coordinate mapping: CropBox when present, otherwise MediaBox.
    pub page_box: BBox,
}

/// Plain RGBA pixels produced by one render call.
#[derive(Debug, Clone, PartialEq)]
pub struct RenderedPage {
    pub width_px: u32,
    pub height_px: u32,
    pub pixels_rgba: Vec<u8>,
}

/// The viewer's renderer interface.
///
/// Intent and paths go in; plain page metadata or pixels come out. There is
/// exactly one implementation: [`PdfiumRenderer`].
pub trait PageRenderer: Send + Sync {
    fn page_count(&self, pdf_path: &Path) -> Result<u32>;

    fn page_info(&self, pdf_path: &Path, page_number: u32) -> Result<PageInfo>;

    fn render_page(
        &self,
        pdf_path: &Path,
        page_number: u32,
        width_px: u32,
        height_px: u32,
    ) -> Result<RenderedPage>;
}

/// Absolute path of the library shipped next to the running executable.
///
/// There is deliberately no system-library fallback: a missing bundled file
/// is an error, not permission to load an unrelated PDFium from the machine.
pub fn bundled_library_path() -> Result<PathBuf> {
    let executable = std::env::current_exe()
        .map_err(|error| DhError::State(format!("cannot locate the app directory: {error}")))?;
    let directory = executable.parent().ok_or_else(|| {
        DhError::State(format!(
            "the app executable '{}' has no parent directory",
            executable.display()
        ))
    })?;
    Ok(directory.join(platform_library_name()))
}

fn platform_library_name() -> OsString {
    Pdfium::pdfium_platform_library_name()
}

static PDFIUM: OnceLock<std::result::Result<Arc<Pdfium>, String>> = OnceLock::new();

/// Bind once to the vendored library, shared by rendering and text extraction.
pub(crate) fn bind_pdfium(library_path: &Path) -> Result<Arc<Pdfium>> {
    if !library_path.is_file() {
        return Err(DhError::State(format!(
            "bundled PDFium library is missing: '{}'",
            library_path.display()
        )));
    }
    PDFIUM
        .get_or_init(|| {
            Pdfium::bind_to_library(library_path)
                .map(|bindings| Arc::new(Pdfium::new(bindings)))
                .map_err(|error| format!("cannot load bundled PDFium: {error}"))
        })
        .as_ref()
        .map_err(|message| DhError::State(message.clone()))
        .cloned()
}

/// The bundled PDFium renderer.
#[derive(Clone)]
pub struct PdfiumRenderer {
    pdfium: Arc<Pdfium>,
}

impl PdfiumRenderer {
    /// Bind to the library co-located with the running executable.
    pub fn load_bundled() -> Result<Self> {
        Self::with_library(&bundled_library_path()?)
    }

    /// Bind to an explicit library path. Used by fixture tests, which point at
    /// the same vendored binary that the app copies next to its executable.
    pub fn with_library(library_path: &Path) -> Result<Self> {
        Ok(Self {
            pdfium: bind_pdfium(library_path)?,
        })
    }

    /// Opens a PDF and returns plain-data diagnostics without changing the
    /// viewer's rendering or the digital extraction path.
    ///
    /// File size and page count are always reported for successfully opened
    /// documents. No size or page-count threshold is applied.
    pub fn open_with_diagnostics(&self, pdf_path: &Path) -> Result<PdfOpenDiagnostics> {
        let document = match self.pdfium.load_pdf_from_file(pdf_path, None) {
            Ok(document) => document,
            Err(error) => {
                return Ok(PdfOpenDiagnostics {
                    diagnostics: vec![load_error_diagnostic(error)],
                    measurements: None,
                });
            }
        };

        let page_count = u32::try_from(document.pages().len())
            .map_err(|_| DhError::Parse("PDF has too many pages".to_owned()))?;
        let size_bytes = std::fs::metadata(pdf_path)
            .map_err(|error| DhError::State(format!("cannot read PDF metadata: {error}")))?
            .len();

        let mut diagnostics = Vec::new();
        for page_number in 1..=page_count {
            let page = page_from(&document, page_number)?;
            let has_positioned_text_runs = !page.text().map_err(pdf_error)?.is_empty();
            let has_image_objects = page
                .objects()
                .iter()
                .any(|object| object.as_image_object().is_some());

            if !has_positioned_text_runs && has_image_objects {
                diagnostics.push(crate::diagnostics::PdfDiagnostic::image_only(page_number));
            } else if !has_positioned_text_runs {
                diagnostics.push(crate::diagnostics::PdfDiagnostic::no_text(page_number));
            }
        }

        Ok(PdfOpenDiagnostics {
            diagnostics,
            measurements: Some(PdfMeasurements {
                size_bytes,
                page_count,
            }),
        })
    }
}

pub(crate) fn pdf_error(error: PdfiumError) -> DhError {
    DhError::Parse(format!("PDFium could not process the PDF: {error}"))
}

fn page_from<'a>(document: &PdfDocument<'a>, page_number: u32) -> Result<PdfPage<'a>> {
    if page_number == 0 {
        return Err(DhError::InvalidInput("page numbers start at 1".to_owned()));
    }
    let index = i32::try_from(page_number - 1)
        .map_err(|_| DhError::InvalidInput("page number is too large".to_owned()))?;
    document.pages().get(index).map_err(pdf_error)
}

pub(crate) fn effective_page_box(page: &PdfPage<'_>) -> Result<BBox> {
    let bounds = page
        .boundaries()
        .crop()
        .or_else(|_| page.boundaries().media())
        .map_err(pdf_error)?
        .bounds;
    Ok(BBox {
        x0: f64::from(bounds.left().value),
        y0: f64::from(bounds.bottom().value),
        x1: f64::from(bounds.right().value),
        y1: f64::from(bounds.top().value),
    })
}

impl PageRenderer for PdfiumRenderer {
    fn page_count(&self, pdf_path: &Path) -> Result<u32> {
        let document = self
            .pdfium
            .load_pdf_from_file(pdf_path, None)
            .map_err(pdf_error)?;
        u32::try_from(document.pages().len())
            .map_err(|_| DhError::Parse("PDF has too many pages".to_owned()))
    }

    fn page_info(&self, pdf_path: &Path, page_number: u32) -> Result<PageInfo> {
        let document = self
            .pdfium
            .load_pdf_from_file(pdf_path, None)
            .map_err(pdf_error)?;
        let page = page_from(&document, page_number)?;
        let rotation_degrees = page.rotation().map_err(pdf_error)?.as_degrees() as u16;
        Ok(PageInfo {
            page_count: u32::try_from(document.pages().len())
                .map_err(|_| DhError::Parse("PDF has too many pages".to_owned()))?,
            page_number,
            width_points: f64::from(page.width().value),
            height_points: f64::from(page.height().value),
            rotation_degrees,
            page_box: effective_page_box(&page)?,
        })
    }

    fn render_page(
        &self,
        pdf_path: &Path,
        page_number: u32,
        width_px: u32,
        height_px: u32,
    ) -> Result<RenderedPage> {
        if width_px == 0 || height_px == 0 {
            return Err(DhError::InvalidInput(
                "render dimensions must be positive".to_owned(),
            ));
        }
        let document = self
            .pdfium
            .load_pdf_from_file(pdf_path, None)
            .map_err(pdf_error)?;
        let page = page_from(&document, page_number)?;
        let rotation = Some(page.rotation().map_err(pdf_error)?);
        let render_width = i32::try_from(width_px)
            .map_err(|_| DhError::InvalidInput("render width is too large".to_owned()))?;
        let render_height = i32::try_from(height_px)
            .map_err(|_| DhError::InvalidInput("render height is too large".to_owned()))?;
        let bitmap = page
            .render(render_width, render_height, rotation)
            .map_err(pdf_error)?;
        Ok(RenderedPage {
            width_px: u32::try_from(bitmap.width()).map_err(|_| {
                DhError::Parse("PDFium returned a negative bitmap width".to_owned())
            })?,
            height_px: u32::try_from(bitmap.height()).map_err(|_| {
                DhError::Parse("PDFium returned a negative bitmap height".to_owned())
            })?,
            pixels_rgba: bitmap.as_rgba_bytes(),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Recorded from `pdfinfo -box -f 1 -l 1` (independent oracle) on the owner
    // fixtures. `pdftoppm -r 72` renders page 1 of each at 596 x 843 px.
    const FIXTURES: &[(&str, u32, f64, f64)] = &[
        ("01.pdf", 6, 595.5, 842.25),
        ("02.pdf", 7, 595.5, 842.25),
        ("03.pdf", 5, 595.5, 842.25),
    ];
    const RENDER_WIDTH_PX: u32 = 596;
    const RENDER_HEIGHT_PX: u32 = 843;

    fn vendored_library() -> PathBuf {
        Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../../vendor/pdfium")
            .join(bundled_library_platform_directory())
            .join(platform_library_name())
    }

    #[cfg(target_os = "windows")]
    fn bundled_library_platform_directory() -> &'static str {
        "win-x64"
    }

    #[cfg(not(target_os = "windows"))]
    fn bundled_library_platform_directory() -> &'static str {
        "linux-x64"
    }

    fn renderer() -> PdfiumRenderer {
        PdfiumRenderer::with_library(&vendored_library()).expect("load vendored PDFium")
    }

    fn owner_pdf(name: &str) -> PathBuf {
        Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("..")
            .join("..")
            .join("PDF_tests")
            .join(name)
    }

    #[test]
    fn bundled_library_path_uses_the_executable_directory() {
        let path = bundled_library_path().expect("app directory library path");
        assert_eq!(path.file_name(), Some(platform_library_name().as_os_str()));
        assert!(path.is_absolute(), "library path must be absolute");
    }

    #[test]
    fn each_owner_pdf_page_one_matches_the_independent_oracle_and_renders_non_blank() {
        let renderer = renderer();
        for (name, expected_pages, expected_width, expected_height) in FIXTURES {
            let path = owner_pdf(name);
            assert!(
                path.is_file(),
                "owner fixture is missing: {}",
                path.display()
            );

            assert_eq!(
                renderer
                    .page_count(&path)
                    .unwrap_or_else(|error| panic!("page count for {name}: {error}")),
                *expected_pages,
                "page count for {name}"
            );
            let info = renderer
                .page_info(&path, 1)
                .unwrap_or_else(|error| panic!("page info for {name}: {error}"));
            assert_eq!(
                info.page_count, *expected_pages,
                "page count in info for {name}"
            );
            assert_eq!(info.page_number, 1);
            assert_eq!(info.width_points, *expected_width, "width for {name}");
            assert_eq!(info.height_points, *expected_height, "height for {name}");
            assert_eq!(info.rotation_degrees, 0, "rotation for {name}");
            assert_eq!(
                info.page_box,
                BBox {
                    x0: 0.0,
                    y0: 0.0,
                    x1: *expected_width,
                    y1: *expected_height,
                },
                "page box for {name}"
            );

            let page = renderer
                .render_page(&path, 1, RENDER_WIDTH_PX, RENDER_HEIGHT_PX)
                .unwrap_or_else(|error| panic!("render page one for {name}: {error}"));
            assert_eq!(
                (page.width_px, page.height_px),
                (RENDER_WIDTH_PX, RENDER_HEIGHT_PX),
                "render dimensions for {name}"
            );
            assert_eq!(
                page.pixels_rgba.len(),
                (RENDER_WIDTH_PX * RENDER_HEIGHT_PX * 4) as usize,
                "RGBA length for {name}"
            );
            let non_white = page
                .pixels_rgba
                .as_chunks::<4>()
                .0
                .iter()
                .filter(|rgba| rgba.iter().any(|channel| *channel < 250))
                .count();
            assert!(non_white > 1000, "{name} rendered a blank page");
        }
    }

    #[test]
    fn missing_library_and_invalid_page_are_reported() {
        assert!(PdfiumRenderer::with_library(Path::new("does-not-exist")).is_err());
        let renderer = renderer();
        let path = owner_pdf("01.pdf");
        assert!(renderer.page_info(&path, 0).is_err());
        assert!(renderer.page_info(&path, 7).is_err());
        assert!(renderer.render_page(&path, 1, 0, 100).is_err());
    }
}
