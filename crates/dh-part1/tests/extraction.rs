//! Stage 3 extraction tests.
//!
//! The owner PDFs in `PDF_tests/` provide the regression corpus. Their stored
//! baseline is PDFium output and is explicitly NOT independent ground truth.

use std::{
    collections::HashSet,
    fs,
    path::{Path, PathBuf},
};

use dh_core::grid::{self, ReconstructionOptions};
use dh_ir::{PageTextExtractor, PositionedTextRun, ReconstructedPage};
use dh_part1::text::PdfiumTextExtractor;
use pdfium_render::prelude::Pdfium;

const FIXTURES: &[(&str, u32)] = &[("01.pdf", 6), ("02.pdf", 7), ("03.pdf", 5)];

fn repository_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("..").join("..")
}

fn vendored_library() -> PathBuf {
    repository_root()
        .join("vendor/pdfium")
        .join(if cfg!(windows) {
            "win-x64"
        } else {
            "linux-x64"
        })
        .join(Pdfium::pdfium_platform_library_name())
}

fn owner_pdf(name: &str) -> PathBuf {
    repository_root().join("PDF_tests").join(name)
}

fn baseline_path(name: &str) -> PathBuf {
    repository_root()
        .join("tests/fixtures/baseline")
        .join(name.replace(".pdf", ".baseline"))
}

fn extractor() -> PdfiumTextExtractor {
    PdfiumTextExtractor::with_library(&vendored_library()).expect("load vendored PDFium")
}

fn options() -> ReconstructionOptions {
    ReconstructionOptions {
        extractor_name: "pdfium-digital-v1".to_owned(),
        extractor_version: "0.9.6+chromium-8125".to_owned(),
        ..ReconstructionOptions::default()
    }
}

fn escape(value: &str) -> String {
    value
        .replace('\\', "\\\\")
        .replace('\t', "\\t")
        .replace('\n', "\\n")
        .replace('\r', "\\r")
}

fn number(value: f64) -> String {
    format!("{value:.3}")
}

fn box_text(bbox: dh_ir::BBox) -> String {
    format!(
        "{},{},{},{}",
        number(bbox.x0),
        number(bbox.y0),
        number(bbox.x1),
        number(bbox.y1)
    )
}

fn serialize_run(run: &PositionedTextRun) -> String {
    let assigned = format!(
        "font='{}' size={} missing=[{}]",
        run.font_name,
        number(run.font_size_points),
        run.missing_font_metadata.join("|")
    );
    let sequence = run
        .order
        .sequence_index
        .map_or_else(|| "<none>".to_owned(), |index| index.to_string());
    let order = format!("{:?}", run.order.order);
    format!(
        "RUN\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}",
        run.run_id,
        escape(&run.text),
        box_text(run.bbox),
        run.font_weight.as_deref().unwrap_or("<none>"),
        run.italic,
        run.bold,
        number(f64::from(run.angle_degrees)),
        sequence,
        order,
        assigned,
        run.font.unavailable_fields.join("|")
    )
}

fn serialize_page_text(
    extraction: &dh_ir::PageTextExtraction,
    reconstructed: &ReconstructedPage,
) -> String {
    let mut lines = vec![format!(
        "PAGE\t{}\tBOX\t{}\tROTATION\t{}\tUNSUPPORTED\t{}",
        extraction.page,
        box_text(extraction.page_box),
        extraction.rotation_degrees,
        extraction
            .unsupported_reason
            .as_deref()
            .map(escape)
            .unwrap_or_else(|| "<none>".to_owned())
    )];
    lines.extend(extraction.runs.iter().map(serialize_run));
    for unassigned in &reconstructed.unassigned_text {
        lines.push(format!(
            "UNASSIGNED\t{}\t{}\t{}\t{}",
            unassigned.run_id,
            escape(&unassigned.text),
            box_text(unassigned.bbox),
            unassigned.reason
        ));
    }
    for warning in &reconstructed.warnings {
        lines.push(format!(
            "WARNING\t{}\t{}\t{}\t{}",
            warning.page,
            box_text(warning.bbox),
            warning.reason,
            escape(&warning.message)
        ));
    }
    for (block_index, block) in reconstructed.blocks.iter().enumerate() {
        lines.push(format!(
            "BLOCK\t{}\t{}\tROWS\t{}",
            block_index,
            box_text(block.bbox),
            block.rows.len()
        ));
        for (row_index, row) in block.rows.iter().enumerate() {
            for cell in row {
                let numeric = cell.numeric.as_ref().map_or_else(
                    || "<none>".to_owned(),
                    |numeric| format!("{}:{:?}", numeric.canonical, numeric.source_digit_script),
                );
                let guard = format!("{:?}", cell.guard);
                lines.push(format!(
                    "CELL\t{}\t{}\t{:?}\t{}\t{}\t{}\t{}\t{}",
                    block_index,
                    row_index,
                    cell.grid_position,
                    box_text(cell.bbox.expect("cell bbox")),
                    escape(cell.text.as_deref().unwrap_or("")),
                    numeric,
                    guard,
                    cell.provenance.run_ids.join("|")
                ));
            }
        }
    }
    lines.join("\n")
}

fn report(label: &str, extraction: &dh_ir::PageTextExtraction, reconstructed: &ReconstructedPage) {
    let assigned: HashSet<&str> = reconstructed
        .blocks
        .iter()
        .flat_map(|block| &block.rows)
        .flatten()
        .flat_map(|cell| cell.provenance.run_ids.iter().map(String::as_str))
        .collect();
    eprintln!(
        "{label}: runs={} assigned={} unassigned={} warnings={}",
        extraction.runs.len(),
        assigned.len(),
        reconstructed.unassigned_text.len(),
        reconstructed.warnings.len()
    );
}

#[test]
fn owner_pdf_baselines_match_provisional_pdfium_output() {
    let extractor = extractor();
    let options = options();
    for (name, expected_pages) in FIXTURES {
        let path = owner_pdf(name);
        assert!(path.is_file(), "owner fixture missing: {}", path.display());
        assert_eq!(
            extractor.page_count(&path).expect("page count"),
            *expected_pages
        );
        let mut serialized = String::new();
        for page in 1..=*expected_pages {
            let extraction = extractor.extract_page(&path, page).expect("extract page");
            assert!(
                extraction.unsupported_reason.is_none(),
                "owner page {page} unexpectedly has no digital text"
            );
            let reconstructed =
                grid::reconstruct_page(page, &extraction.runs, &options).expect("reconstruct page");
            report(&format!("{name} p{page}"), &extraction, &reconstructed);
            serialized.push_str(&serialize_page_text(&extraction, &reconstructed));
            serialized.push('\n');
        }
        let baseline = baseline_path(name);
        if std::env::var_os("DH_UPDATE_EXTRACTION_BASELINE").is_some() {
            fs::create_dir_all(baseline.parent().expect("baseline parent"))
                .expect("create baseline dir");
            fs::write(&baseline, &serialized).expect("write baseline");
        }
        let stored = fs::read_to_string(&baseline).unwrap_or_else(|error| {
            panic!("baseline {} is missing ({error}); this is provisional PDFium output, not ground truth", baseline.display())
        });
        assert_eq!(
            serialized, stored,
            "provisional baseline mismatch for {name}"
        );
    }
}

#[test]
fn every_owner_pdf_run_is_accounted_for_exactly_once() {
    let extractor = extractor();
    let options = options();
    for (name, _) in FIXTURES {
        for page in 1..=extractor.page_count(&owner_pdf(name)).expect("page count") {
            let extraction = extractor
                .extract_page(&owner_pdf(name), page)
                .expect("extract page");
            let reconstructed =
                grid::reconstruct_page(page, &extraction.runs, &options).expect("reconstruct page");
            let mut seen = HashSet::new();
            for block in &reconstructed.blocks {
                for row in &block.rows {
                    for cell in row {
                        for id in &cell.provenance.run_ids {
                            assert!(
                                seen.insert(id.clone()),
                                "{name} p{page}: run {id} appears twice"
                            );
                        }
                    }
                }
            }
            for unassigned in &reconstructed.unassigned_text {
                assert!(
                    seen.insert(unassigned.run_id.clone()),
                    "{name} p{page}: unassigned run appears twice"
                );
            }
            assert_eq!(
                seen.len(),
                extraction.runs.len(),
                "{name} p{page}: a run was silently dropped"
            );
        }
    }
}

/// Minimal PDF with a 1x1 image XObject and no text operators. This is a
/// behavioural fixture only; it is not used as extraction accuracy evidence.
fn image_only_pdf() -> Vec<u8> {
    let objects = [
        b"<< /Type /Catalog /Pages 2 0 R >>".to_vec(),
        b"<< /Type /Pages /Kids [3 0 R] /Count 1 >>".to_vec(),
        b"<< /Type /Page /Parent 2 0 R /MediaBox [0 0 100 100] /Resources << /XObject << /Im0 4 0 R >> >> /Contents 5 0 R >>".to_vec(),
        b"<< /Type /XObject /Subtype /Image /Width 1 /Height 1 /ColorSpace /DeviceGray /BitsPerComponent 8 /Length 1 >>\nstream\n\x00\nendstream".to_vec(),
        b"<< /Length 44 >>\nstream\nq 100 0 0 100 0 0 cm /Im0 Do Q\nendstream".to_vec(),
    ];
    let mut pdf = b"%PDF-1.4\n".to_vec();
    let mut offsets = Vec::new();
    for (index, object) in objects.iter().enumerate() {
        offsets.push(pdf.len());
        pdf.extend_from_slice(format!("{} 0 obj\n", index + 1).as_bytes());
        pdf.extend_from_slice(object);
        pdf.extend_from_slice(b"\nendobj\n");
    }
    let xref = pdf.len();
    pdf.extend_from_slice(format!("xref\n0 {}\n", objects.len() + 1).as_bytes());
    pdf.extend_from_slice(b"0000000000 65535 f \n");
    for offset in offsets {
        pdf.extend_from_slice(format!("{offset:010} 00000 n \n").as_bytes());
    }
    pdf.extend_from_slice(
        format!(
            "trailer\n<< /Size {} /Root 1 0 R >>\nstartxref\n{xref}\n%%EOF\n",
            objects.len() + 1
        )
        .as_bytes(),
    );
    pdf
}

#[test]
fn image_only_page_is_reported_as_unsupported_without_crashing() {
    let directory = std::env::temp_dir().join(format!("dh-s3-image-only-{}", std::process::id()));
    fs::create_dir_all(&directory).expect("create fixture directory");
    let path = directory.join("image-only.pdf");
    fs::write(&path, image_only_pdf()).expect("write behavioural PDF");
    let result = extractor().extract_page(&path, 1);
    let _ = fs::remove_dir_all(&directory);
    let extraction = result.expect("no-text page must be an unsupported result, not an error");
    assert!(extraction.runs.is_empty());
    assert!(extraction
        .unsupported_reason
        .as_deref()
        .is_some_and(|reason| reason.contains("unsupported")));
}
