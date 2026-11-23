//! Viewer-only page rendering through the bundled PDFium library.
//!
//! This is the single renderer implementation used by the desktop viewer. It
//! never supplies pixels to extraction; digital text extraction must use the
//! PDF's own text data. A document is opened for the duration of one call so
//! the renderer stays `Send + Sync` and the Slint UI thread only ever receives
//! plain pixel data.

use std::{
    path::{Path, PathBuf},
    sync::{Arc, OnceLock},
};

use dh_ir::{BBox, DhError, Result};
use pdfium_render::prelude::*;

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
    Ok(directory.join(Pdfium::pdfium_platform_library_name()))
}

static PDFIUM: OnceLock<Arc<Pdfium>> = OnceLock::new();

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

    /// Bind to an explicit library path. Used by smoke tests, which point at
    /// the same vendored binary that the app copies next to its executable.
    pub fn with_library(library_path: &Path) -> Result<Self> {
        if !library_path.is_file() {
            return Err(DhError::State(format!(
                "bundled PDFium library is missing: '{}'",
                library_path.display()
            )));
        }
        let pdfium = PDFIUM
            .get_or_init(|| {
                // The binding is a process-wide once cell. If a test or the app
                // reached this point, the pinned vendored library is the only
                // path that can have initialized it.
                let bindings = Pdfium::bind_to_library(library_path)
                    .expect("the pinned bundled PDFium library must bind");
                Arc::new(Pdfium::new(bindings))
            })
            .clone();
        Ok(Self { pdfium })
    }
}

fn pdf_error(error: PdfiumError) -> DhError {
    DhError::Parse(format!("PDFium could not process the PDF: {error}"))
}

fn page_from<'a>(
    document: &'a PdfDocument<'a>,
    page_number: u32,
) -> Result<PdfPage<'a>> {
    if page_number == 0 {
        return Err(DhError::InvalidInput(
            "page numbers start at 1".to_owned(),
        ));
    }
    document
        .pages()
        .get((page_number - 1) as usize)
        .map_err(pdf_error)
}

fn effective_page_box(page: &PdfPage<'_>) -> Result<BBox> {
    let bounds = page
        .boundaries()
        .crop()
        .or_else(|_| page.boundaries().media())
        .map_err(pdf_error)?
        .bounds;
    Ok(BBox {
        x0: f64::from(bounds.left.value),
        y0: f64::from(bounds.bottom.value),
        x1: f64::from(bounds.right.value),
        y1: f64::from(bounds.top.value),
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
        let bitmap = page.render(width_px, height_px, rotation).map_err(pdf_error)?;
        Ok(RenderedPage {
            width_px: bitmap.width(),
            height_px: bitmap.height(),
            pixels_rgba: bitmap.as_rgba_bytes(),
        })
    }
}

/// Generates a minimal digital PDF for renderer smoke tests and the app's
/// diagnostic self-test. The page is 200 x 100 points and contains two opaque
/// filled rectangles, so a successful render is never blank.
///
/// This is a fixture generator, not ground truth for extraction accuracy.
pub fn generated_smoke_pdf() -> Vec<u8> {
    let content = b"1 0 0 rg\n10 10 80 40 re f\n0 0 1 rg\n110 20 60 60 re f\n";
    let objects = [
        b"<< /Type /Catalog /Pages 2 0 R >>".to_vec(),
        b"<< /Type /Pages /Kids [3 0 R] /Count 1 >>".to_vec(),
        b"<< /Type /Page /Parent 2 0 R /MediaBox [0 0 200 100] /Resources << >> /Contents 4 0 R >>"
            .to_vec(),
        [
            b"<< /Length ".as_slice(),
            content.len().to_string().as_bytes(),
            b" >>\nstream\n",
            content.as_slice(),
            b"endstream",
        ]
        .concat(),
    ];

    let mut bytes = b"%PDF-1.4\n".to_vec();
    let mut offsets = Vec::with_capacity(objects.len());
    for (index, object) in objects.iter().enumerate() {
        offsets.push(bytes.len());
        bytes.extend_from_slice(format!("{} 0 obj\n", index + 1).as_bytes());
        bytes.extend_from_slice(object);
        bytes.extend_from_slice(b"\nendobj\n");
    }
    let xref_offset = bytes.len();
    bytes.extend_from_slice(format!("xref\n0 {}\n", objects.len() + 1).as_bytes());
    bytes.extend_from_slice(b"0000000000 65535 f \n");
    for offset in offsets {
        bytes.extend_from_slice(format!("{offset:010} 00000 n \n").as_bytes());
    }
    bytes.extend_from_slice(
        format!(
            "trailer\n<< /Size {} /Root 1 0 R >>\nstartxref\n{}\n%%EOF\n",
            objects.len() + 1,
            xref_offset
        )
        .as_bytes(),
    );
    bytes
}

#[cfg(test)]
mod tests {
    use super::*;

    fn vendored_library() -> PathBuf {
        Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../../vendor/pdfium")
            .join(bundled_library_platform_directory())
            .join(Pdfium::pdfium_platform_library_name())
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

    fn smoke_pdf_path() -> PathBuf {
        let path = std::env::temp_dir().join(format!(
            "dh-pdfium-smoke-{}-{}.pdf",
            std::process::id(),
            std::thread::current().name().unwrap_or("test")
        ));
        std::fs::write(&path, generated_smoke_pdf()).expect("write generated smoke PDF");
        path
    }

    #[test]
    fn bundled_library_path_uses_the_executable_directory() {
        let path = bundled_library_path().expect("app directory library path");
        assert_eq!(
            path.file_name(),
            Some(Pdfium::pdfium_platform_library_name().as_os_str())
        );
        assert!(path.is_absolute(), "library path must be absolute");
    }

    #[test]
    fn render_page_one_has_expected_dimensions_and_is_not_blank() {
        let path = smoke_pdf_path();
        let page = renderer()
            .render_page(&path, 1, 200, 100)
            .expect("render generated page one");
        assert_eq!((page.width_px, page.height_px), (200, 100));
        assert_eq!(page.pixels_rgba.len(), 200 * 100 * 4);
        assert!(
            page.pixels_rgba.chunks_exact(4).any(|rgba| rgba[0] > 128),
            "the red rectangle must produce non-blank pixels"
        );
        std::fs::remove_file(path).expect("remove generated smoke PDF");
    }

    #[test]
    fn page_metadata_exposes_count_size_rotation_and_page_box() {
        let path = smoke_pdf_path();
        let renderer = renderer();
        assert_eq!(renderer.page_count(&path).expect("page count"), 1);
        let info = renderer.page_info(&path, 1).expect("page info");
        assert_eq!(info.page_count, 1);
        assert_eq!(info.page_number, 1);
        assert_eq!((info.width_points, info.height_points), (200.0, 100.0));
        assert_eq!(info.rotation_degrees, 0);
        assert_eq!(
            info.page_box,
            BBox {
                x0: 0.0,
                y0: 0.0,
                x1: 200.0,
                y1: 100.0
            }
        );
        std::fs::remove_file(path).expect("remove generated smoke PDF");
    }

    #[test]
    fn missing_library_and_invalid_page_are_reported() {
        assert!(PdfiumRenderer::with_library(Path::new("does-not-exist")).is_err());
        let path = smoke_pdf_path();
        let renderer = renderer();
        assert!(renderer.page_info(&path, 0).is_err());
        assert!(renderer.page_info(&path, 2).is_err());
        assert!(renderer.render_page(&path, 1, 0, 100).is_err());
        std::fs::remove_file(path).expect("remove generated smoke PDF");
    }
}
