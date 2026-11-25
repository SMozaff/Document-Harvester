//! Plain-data diagnostics for opening digital PDFs.
//!
//! Diagnostics are returned to callers; this module performs no UI updates,
//! persistence, OCR, or size-limit enforcement.

use pdfium_render::prelude::*;

/// Stable machine-readable diagnostic identifiers.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DiagnosticCode {
    Encrypted,
    Corrupt,
    ImageOnly,
    NoText,
}

impl DiagnosticCode {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Encrypted => "encrypted",
            Self::Corrupt => "corrupt",
            Self::ImageOnly => "image_only",
            Self::NoText => "no_text",
        }
    }
}

/// One diagnostic, with a page number for page-scoped findings.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PdfDiagnostic {
    pub code: DiagnosticCode,
    pub message: String,
    pub page_number: Option<u32>,
}

impl PdfDiagnostic {
    pub(crate) fn load_failure(code: DiagnosticCode, message: &'static str) -> Self {
        Self {
            code,
            message: message.to_owned(),
            page_number: None,
        }
    }

    pub(crate) fn image_only(page_number: u32) -> Self {
        Self {
            code: DiagnosticCode::ImageOnly,
            message: "Page contains image objects but no positioned digital text runs."
                .to_owned(),
            page_number: Some(page_number),
        }
    }

    pub(crate) fn no_text(page_number: u32) -> Self {
        Self {
            code: DiagnosticCode::NoText,
            message: "Page contains neither positioned digital text runs nor image objects."
                .to_owned(),
            page_number: Some(page_number),
        }
    }
}

/// Measurements for a successfully opened PDF.
///
/// These values are informational only. No maximum file size or page count is
/// defined or enforced by the diagnostics interface.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PdfMeasurements {
    pub size_bytes: u64,
    pub page_count: u32,
}

/// Result of the PDF open-time diagnostic pass.
///
/// A load failure has one document-level diagnostic and no page measurements.
/// A successfully opened document always has measurements, even when the
/// values are large; the implementation does not define an "oversized" limit.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PdfOpenDiagnostics {
    pub diagnostics: Vec<PdfDiagnostic>,
    pub measurements: Option<PdfMeasurements>,
}

pub(crate) fn load_error_diagnostic(error: PdfiumError) -> PdfDiagnostic {
    match error {
        PdfiumError::PdfiumLibraryInternalError(PdfiumInternalError::PasswordError) => {
            PdfDiagnostic::load_failure(
                DiagnosticCode::Encrypted,
                "PDF is password-protected and could not be opened.",
            )
        }
        _ => PdfDiagnostic::load_failure(
            DiagnosticCode::Corrupt,
            "PDF could not be opened because it is unreadable or has an unsupported format.",
        ),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::renderer::{bundled_library_path, PdfiumRenderer};
    use std::{
        fs,
        path::{Path, PathBuf},
        sync::atomic::{AtomicU64, Ordering},
    };

    static NEXT_TEMP_FILE: AtomicU64 = AtomicU64::new(0);

    struct TempPdf(PathBuf);

    impl TempPdf {
        fn new(bytes: &[u8]) -> Self {
            let sequence = NEXT_TEMP_FILE.fetch_add(1, Ordering::Relaxed);
            let path = std::env::temp_dir().join(format!(
                "document-harvester-diagnostics-{}-{sequence}.pdf",
                std::process::id()
            ));
            fs::write(&path, bytes).expect("write temporary PDF fixture");
            Self(path)
        }

        fn path(&self) -> &Path {
            &self.0
        }
    }

    impl Drop for TempPdf {
        fn drop(&mut self) {
            let _ = fs::remove_file(&self.0);
        }
    }

    fn renderer() -> PdfiumRenderer {
        PdfiumRenderer::with_library(&bundled_library_path().expect("bundled PDFium path"))
            .expect("bind bundled PDFium")
    }

    // Constructed directly from known PDF objects. This deliberately avoids
    // using the diagnostics implementation to decide the expected page type.
    fn page_objects(image_only: bool) -> Vec<Vec<u8>> {
        let page_resources = if image_only {
            "<< /XObject << /Im0 5 0 R >> >>"
        } else {
            "<< >>"
        };
        let content = if image_only {
            b"q 100 0 0 100 0 0 cm /Im0 Do Q".as_slice()
        } else {
            b"".as_slice()
        };

        let mut objects = vec![
            b"<< /Type /Catalog /Pages 2 0 R >>".to_vec(),
            b"<< /Type /Pages /Kids [3 0 R] /Count 1 >>".to_vec(),
            format!(
                "<< /Type /Page /Parent 2 0 R /MediaBox [0 0 100 100] /Resources {page_resources} /Contents 4 0 R >>"
            )
            .into_bytes(),
        ];

        let mut content_object = format!("<< /Length {} >>\\nstream\\n", content.len()).into_bytes();
        content_object.extend_from_slice(content);
        content_object.extend_from_slice(b"\\nendstream");
        objects.push(content_object);

        if image_only {
            let mut image_object = b"<< /Type /XObject /Subtype /Image /Width 1 /Height 1 /ColorSpace /DeviceRGB /BitsPerComponent 8 /Length 3 >>\\nstream\\n".to_vec();
            image_object.extend_from_slice(&[0, 0, 0]);
            image_object.extend_from_slice(b"\\nendstream");
            objects.push(image_object);
        }
        objects
    }

    fn assemble_pdf(objects: Vec<Vec<u8>>, trailer_entries: &str) -> Vec<u8> {
        let mut pdf = b"%PDF-1.4\\n".to_vec();
        let mut offsets = Vec::new();
        for (index, object) in objects.iter().enumerate() {
            offsets.push(pdf.len());
            pdf.extend_from_slice(format!("{} 0 obj\\n", index + 1).as_bytes());
            pdf.extend_from_slice(object);
            pdf.extend_from_slice(b"\\nendobj\\n");
        }
        let xref_offset = pdf.len();
        pdf.extend_from_slice(format!("xref\\n0 {}\\n", objects.len() + 1).as_bytes());
        pdf.extend_from_slice(b"0000000000 65535 f \\n");
        for offset in offsets {
            pdf.extend_from_slice(format!("{offset:010} 00000 n \\n").as_bytes());
        }
        pdf.extend_from_slice(
            format!(
                "trailer\\n<< /Size {} /Root 1 0 R {trailer_entries} >>\\nstartxref\\n{xref_offset}\\n%%EOF\\n",
                objects.len() + 1
            )
            .as_bytes(),
        );
        pdf
    }

    fn one_page_pdf(image_only: bool) -> Vec<u8> {
        assemble_pdf(page_objects(image_only), "")
    }

    // A valid Standard Security Handler revision-2 PDF with a non-empty user
    // password ("secret"). O and U are fixed independently computed RC4/MD5
    // entries; opening with no password must therefore return PasswordError.
    fn encrypted_pdf() -> Vec<u8> {
        let mut objects = page_objects(false);
        objects.push(
            b"<< /Filter /Standard /V 1 /R 2 /Length 40 /O <92fe0f4454ad4c9644693f33c07cb54f587dce1e2682fe9ecea6107a1ef630dd> /U <f4c3199cb21e35b4bd6c22a97a2acca13b084d9016256900cd312e0758df7e6f00000000000000000000000000000000> /P -4 >>"
                .to_vec(),
        );
        assemble_pdf(
            objects,
            "/Encrypt 5 0 R /ID [<00112233445566778899aabbccddeeff><00112233445566778899aabbccddeeff>]",
        )
    }

    fn has_code(result: &PdfOpenDiagnostics, code: DiagnosticCode) -> bool {
        result.diagnostics.iter().any(|diagnostic| diagnostic.code == code)
    }

    #[test]
    fn encrypted_pdf_returns_encrypted_code() {
        let input = TempPdf::new(&encrypted_pdf());
        let result = renderer()
            .open_with_diagnostics(input.path())
            .expect("diagnostic result");
        assert!(has_code(&result, DiagnosticCode::Encrypted));
    }

    #[test]
    fn password_error_variant_maps_to_encrypted_without_message_inspection() {
        let diagnostic = load_error_diagnostic(PdfiumError::PdfiumLibraryInternalError(
            PdfiumInternalError::PasswordError,
        ));
        assert_eq!(diagnostic.code.as_str(), "encrypted");
        assert_eq!(diagnostic.page_number, None);
    }

    #[test]
    fn truncated_or_garbage_input_returns_corrupt_code() {
        let input = TempPdf::new(b"%PDF-1.7\n1 0 obj\n<< /Type /Catalog >>");
        let result = renderer()
            .open_with_diagnostics(input.path())
            .expect("diagnostic result");
        assert!(has_code(&result, DiagnosticCode::Corrupt));
        assert_eq!(result.measurements, None);
    }

    #[test]
    fn image_only_page_is_reported_with_its_one_based_page_number() {
        let input = TempPdf::new(&one_page_pdf(true));
        let result = renderer()
            .open_with_diagnostics(input.path())
            .expect("diagnostic result");
        let diagnostic = result
            .diagnostics
            .iter()
            .find(|diagnostic| diagnostic.code == DiagnosticCode::ImageOnly)
            .expect("image-only diagnostic");
        assert_eq!(diagnostic.page_number, Some(1));
    }

    #[test]
    fn blank_page_is_no_text_and_not_image_only() {
        let input = TempPdf::new(&one_page_pdf(false));
        let result = renderer()
            .open_with_diagnostics(input.path())
            .expect("diagnostic result");
        let diagnostic = result
            .diagnostics
            .iter()
            .find(|diagnostic| diagnostic.code == DiagnosticCode::NoText)
            .expect("no-text diagnostic");
        assert_eq!(diagnostic.page_number, Some(1));
        assert!(!has_code(&result, DiagnosticCode::ImageOnly));
    }

    #[test]
    fn large_pdf_reports_measurements_without_a_size_limit_error() {
        let source = Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../../PDF_tests/01.pdf");
        let mut bytes = fs::read(&source).expect("read owner PDF");
        let original_len = bytes.len();
        bytes.extend(std::iter::repeat(b' ').take(8 * 1024 * 1024));
        let input = TempPdf::new(&bytes);
        let result = renderer()
            .open_with_diagnostics(input.path())
            .expect("diagnostic result");
        let measurements = result.measurements.expect("PDF measurements");
        assert_eq!(measurements.size_bytes, (original_len + 8 * 1024 * 1024) as u64);
        assert_eq!(measurements.page_count, 6);
        assert!(!result.diagnostics.iter().any(|diagnostic| diagnostic.code.as_str() == "oversized"));
    }

    #[test]
    fn all_three_owner_pdfs_have_no_diagnostics() {
        let renderer = renderer();
        let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../PDF_tests");
        for name in ["01.pdf", "02.pdf", "03.pdf"] {
            let result = renderer
                .open_with_diagnostics(&root.join(name))
                .unwrap_or_else(|error| panic!("diagnostics for {name}: {error}"));
            assert!(
                result.diagnostics.is_empty(),
                "unexpected diagnostics for {name}: {:?}",
                result.diagnostics
            );
            assert!(result.measurements.is_some());
        }
    }
}
