//! Isolated Phase 0 spike for repeated XLSX creation and append.
//!
//! This crate is intentionally not a production dependency of dh-writer.
//! It exists to exercise candidate APIs and independent read-back before
//! making a backend selection.

mod rust_xlsxwriter_fidelity;

#[cfg(test)]
mod tests {
    use std::{
        path::PathBuf,
        time::{SystemTime, UNIX_EPOCH},
    };

    use calamine::{open_workbook, Reader, Xlsx};
    use umya_spreadsheet::{new_file, reader, writer};

    fn temp_xlsx(label: &str) -> PathBuf {
        let nonce = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system clock before Unix epoch")
            .as_nanos();
        std::env::temp_dir().join(format!(
            "document-harvester-{label}-{}-{nonce}.xlsx",
            std::process::id()
        ))
    }

    #[test]
    fn creates_a_blank_workbook_readable_by_an_independent_reader() {
        let path = temp_xlsx("blank");

        let workbook = new_file();
        writer::xlsx::write(&workbook, &path).expect("write blank workbook");

        let mut independent: Xlsx<_> =
            open_workbook(&path).expect("calamine must open generated workbook");
        assert!(independent
            .sheet_names()
            .iter()
            .any(|name| name == "Sheet1"));
        let range = independent
            .worksheet_range("Sheet1")
            .expect("default worksheet must be readable");
        assert_eq!(range.used_cells().count(), 0);

        std::fs::remove_file(path).expect("remove temporary workbook");
    }

    #[test]
    fn reopens_and_appends_consecutive_selections_without_a_spacer_row() {
        let path = temp_xlsx("append");

        // Project creation: one new, blank workbook.
        let workbook = new_file();
        writer::xlsx::write(&workbook, &path).expect("write initial workbook");

        // First approved capture.
        let mut workbook = reader::xlsx::read(&path).expect("reopen blank workbook");
        let sheet = workbook
            .sheet_by_name_mut("Sheet1")
            .expect("default worksheet exists");
        sheet.cell_mut("A1").set_value_string("selection one");
        sheet.cell_mut("B1").set_value_string("فارسی");
        sheet.cell_mut("C1").set_value_string("English / فارسی");
        sheet.cell_mut("D1").set_value_string("می‌رود");
        sheet.cell_mut("E1").set_value_number(123.5_f64);
        writer::xlsx::write(&workbook, &path).expect("write first capture");

        // Second approved capture: reopen the same workbook and append directly
        // to the next row, preserving the first capture.
        let mut workbook = reader::xlsx::read(&path).expect("reopen after first capture");
        let sheet = workbook
            .sheet_by_name_mut("Sheet1")
            .expect("default worksheet exists");
        sheet.cell_mut("A2").set_value_string("selection two");
        sheet.cell_mut("B2").set_value_string("second capture");
        writer::xlsx::write(&workbook, &path).expect("write second capture");

        // Independent read-back with calamine, not the candidate writer itself.
        let mut independent: Xlsx<_> =
            open_workbook(&path).expect("independent reader opens appended workbook");
        let range = independent
            .worksheet_range("Sheet1")
            .expect("independent reader reads worksheet");

        assert_eq!(
            range.get_value((0, 0)).map(ToString::to_string).as_deref(),
            Some("selection one")
        );
        assert_eq!(
            range.get_value((1, 0)).map(ToString::to_string).as_deref(),
            Some("selection two")
        );
        assert_eq!(
            range.get_value((0, 1)).map(ToString::to_string).as_deref(),
            Some("فارسی")
        );
        assert_eq!(
            range.get_value((0, 2)).map(ToString::to_string).as_deref(),
            Some("English / فارسی")
        );
        assert_eq!(
            range.get_value((0, 3)).map(ToString::to_string).as_deref(),
            Some("می‌رود")
        );
        assert_eq!(
            range.get_value((0, 4)).map(ToString::to_string).as_deref(),
            Some("123.5")
        );
        assert_eq!(
            range.get_value((1, 1)).map(ToString::to_string).as_deref(),
            Some("second capture")
        );

        // There is no intentionally inserted blank row between captures.
        assert_eq!(
            range.get_value((1, 0)).unwrap().to_string(),
            "selection two"
        );
        std::fs::remove_file(path).expect("remove temporary workbook");
    }
    #[test]
    fn preserves_formula_style_and_merged_range_across_append() {
        let path = temp_xlsx("features");

        let mut workbook = new_file();
        {
            let sheet = workbook
                .sheet_by_name_mut("Sheet1")
                .expect("default worksheet exists");
            sheet
                .cell_mut("A1")
                .set_formula("SUM(1,2)")
                .set_formula_result_number(3.0_f64);
            sheet
                .style_mut("B1")
                .set_background_color(umya_spreadsheet::Color::COLOR_RED_STR);
            sheet.add_merge_cells("C1:D1");
        }
        writer::xlsx::write(&workbook, &path).expect("write feature fixture");

        // Reopen and append as the application will for a later capture.
        let mut workbook = reader::xlsx::read(&path).expect("reopen feature fixture");
        {
            let sheet = workbook
                .sheet_by_name_mut("Sheet1")
                .expect("default worksheet exists");
            assert_eq!(
                sheet.cell("A1").expect("formula cell exists").formula(),
                "SUM(1,2)"
            );
            assert!(sheet.style("B1").background_color().is_some());
            assert_eq!(sheet.merge_cells().len(), 1);
            sheet
                .cell_mut("A2")
                .set_value_string("appended after features");
        }
        writer::xlsx::write(&workbook, &path).expect("write appended feature fixture");

        // Verify the candidate itself retained the existing workbook features.
        let workbook = reader::xlsx::read(&path).expect("reopen appended feature fixture");
        let sheet = workbook
            .sheet_by_name("Sheet1")
            .expect("default worksheet exists");
        assert_eq!(
            sheet.cell("A1").expect("formula cell exists").formula(),
            "SUM(1,2)"
        );
        assert!(sheet.style("B1").background_color().is_some());
        assert_eq!(sheet.merge_cells().len(), 1);

        // Verify the formula independently as well.
        let mut independent: Xlsx<_> =
            open_workbook(&path).expect("independent reader opens feature fixture");
        let formulas = independent
            .worksheet_formula("Sheet1")
            .expect("independent reader reads formulas");
        assert_eq!(
            formulas.get_value((0, 0)).map(String::as_str),
            Some("SUM(1,2)")
        );

        drop(formulas);
        drop(independent);
        drop(workbook);
        std::fs::remove_file(path).expect("remove temporary workbook");
    }
    #[test]
    fn failed_temporary_write_does_not_change_the_committed_workbook() {
        let path = temp_xlsx("preserve");
        let mut workbook = new_file();
        workbook
            .sheet_by_name_mut("Sheet1")
            .expect("default worksheet exists")
            .cell_mut("A1")
            .set_value_string("committed data");
        writer::xlsx::write(&workbook, &path).expect("write committed workbook");

        let before = std::fs::read(&path).expect("read committed workbook bytes");
        let missing_parent = path
            .parent()
            .expect("temporary path has a parent")
            .join("document-harvester-intentionally-missing-parent")
            .join("pending.xlsx");
        assert!(
            writer::xlsx::write(&workbook, &missing_parent).is_err(),
            "writing into a missing parent directory must fail"
        );

        let after = std::fs::read(&path).expect("read committed workbook after failure");
        assert_eq!(
            after, before,
            "failed temporary write changed committed bytes"
        );

        drop(workbook);
        std::fs::remove_file(path).expect("remove temporary workbook");
    }
}
