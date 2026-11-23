//! Stage 5 task 5.1 fidelity characterization for `rust_xlsxwriter 0.104.0`.
//!
//! Evidence only. These tests describe what the candidate does; they do not
//! select an XLSX backend or define product policy. Every cell is read back
//! with `calamine` and/or inspected in the raw OOXML parts, never with
//! `rust_xlsxwriter` itself.

#[cfg(test)]
mod tests {
    use std::{
        fs::File,
        io::Read,
        path::{Path, PathBuf},
        time::{SystemTime, UNIX_EPOCH},
    };

    use calamine::{open_workbook, Data, Reader, Xlsx};
    use rust_xlsxwriter::{
        Chart, ChartType, ConditionalFormatCell, ConditionalFormatCellRule, DataValidation,
        DataValidationRule, Format, Image, Note, Table, TableColumn, Workbook, XlsxError,
    };

    fn temp_xlsx(label: &str) -> PathBuf {
        let nonce = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system clock before Unix epoch")
            .as_nanos();
        std::env::temp_dir().join(format!(
            "document-harvester-rxw-{label}-{}-{nonce}.xlsx",
            std::process::id()
        ))
    }

    fn ooxml_part(path: &Path, name: &str) -> Option<String> {
        let file = File::open(path).expect("open generated workbook for raw OOXML inspection");
        let mut archive = zip::ZipArchive::new(file).expect("workbook is a readable ZIP");
        let mut entry = archive.by_name(name).ok()?;
        let mut text = String::new();
        entry
            .read_to_string(&mut text)
            .expect("read OOXML part as UTF-8");
        Some(text)
    }

    fn ooxml_part_names(path: &Path) -> Vec<String> {
        let file = File::open(path).expect("open generated workbook for raw OOXML inspection");
        let archive = zip::ZipArchive::new(file).expect("workbook is a readable ZIP");
        archive
            .file_names()
            .filter_map(Result::ok)
            .map(|name| name.into_owned())
            .collect()
    }

    fn independent_value(path: &Path, row: u32, col: u16) -> Data {
        let mut book: Xlsx<_> = open_workbook(path).expect("calamine opens candidate workbook");
        let range = book
            .worksheet_range("Sheet1")
            .expect("calamine reads Sheet1");
        range
            .get_value((row, u32::from(col)))
            .expect("cell exists for independent read-back")
            .clone()
    }

    #[test]
    fn formula_like_and_leading_zero_text_is_written_as_a_string() {
        let path = temp_xlsx("literal-strings");
        let inputs = [
            "=SUM(1,2)",
            "+1+1",
            "-1",
            "@SUM(A1:A2)",
            "00123",
            "  padded",
            "English / فارسی",
        ];

        let mut workbook = Workbook::new();
        let sheet = workbook.add_worksheet();
        for (row, text) in inputs.iter().enumerate() {
            sheet
                .write_string(row as u32, 0, *text)
                .expect("write literal string");
            // The generic write() path is also characterized: in 0.104.0 it
            // delegates &str to store_string(), not to the formula writer.
            sheet
                .write(row as u32, 1, *text)
                .expect("generic write of &str");
        }
        workbook.save(&path).expect("save workbook");

        for (row, text) in inputs.iter().enumerate() {
            assert_eq!(
                independent_value(&path, row as u32, 0),
                Data::String((*text).to_string()),
                "write_string changed cell {row}"
            );
            assert_eq!(
                independent_value(&path, row as u32, 1),
                Data::String((*text).to_string()),
                "generic write changed cell {row}"
            );
        }

        let xml = ooxml_part(&path, "xl/worksheets/sheet1.xml").expect("worksheet part exists");
        assert!(!xml.contains("<f>"), "unexpected formula element: {xml}");
        let strings = ooxml_part(&path, "xl/sharedStrings.xml").unwrap_or_default();
        assert!(
            strings.contains("=SUM(1,2)") && strings.contains("@SUM(A1:A2)"),
            "formula-like text missing from shared strings: {strings}"
        );

        std::fs::remove_file(path).expect("remove temporary workbook");
    }

    #[test]
    fn string_length_limit_is_enforced_in_utf16_units() {
        let path = temp_xlsx("string-limit");
        let mut workbook = Workbook::new();
        let sheet = workbook.add_worksheet();

        sheet
            .write_string(0, 0, "a".repeat(32_767))
            .expect("32,767 ASCII characters are accepted");
        assert!(matches!(
            sheet.write_string(1, 0, "a".repeat(32_768)),
            Err(XlsxError::MaxStringLengthExceeded)
        ));

        // One Persian character is one UTF-16 unit; an emoji is two.
        sheet
            .write_string(2, 0, "😀".repeat(16_383))
            .expect("32,766 UTF-16 units are accepted");
        assert!(matches!(
            sheet.write_string(3, 0, "😀".repeat(16_384)),
            Err(XlsxError::MaxStringLengthExceeded)
        ));

        workbook.save(&path).expect("save workbook");
        assert_eq!(
            independent_value(&path, 0, 0),
            Data::String("a".repeat(32_767))
        );
        std::fs::remove_file(path).expect("remove temporary workbook");
    }

    #[test]
    fn excel_grid_limits_are_enforced() {
        let path = temp_xlsx("grid-limits");
        let mut workbook = Workbook::new();
        let sheet = workbook.add_worksheet();

        sheet
            .write_string(1_048_575, 16_383, "last legal cell")
            .expect("XFD1048576 is the last legal cell");
        assert!(matches!(
            sheet.write_string(1_048_576, 0, "past last row"),
            Err(XlsxError::RowColumnLimitError)
        ));
        assert!(matches!(
            sheet.write_string(0, 16_384, "past last column"),
            Err(XlsxError::RowColumnLimitError)
        ));

        workbook.save(&path).expect("save workbook");
        assert_eq!(
            independent_value(&path, 1_048_575, 16_383),
            Data::String("last legal cell".to_string())
        );
        std::fs::remove_file(path).expect("remove temporary workbook");
    }

    #[test]
    #[allow(clippy::excessive_precision)] // The extra digits are the subject of this test.
    fn numeric_precision_and_leading_zeros_are_characterized() {
        let path = temp_xlsx("precision");
        let mut workbook = Workbook::new();
        let sheet = workbook.add_worksheet();

        sheet
            .write_number(0, 0, 123_456_789_012_345_678.0_f64)
            .expect("write 18 significant digits");
        sheet
            .write_number(1, 0, 0.123_456_789_012_345_678_f64)
            .expect("write high precision fraction");
        sheet
            .write_string(2, 0, "00123")
            .expect("preserve leading zeros as text");
        let text_format = Format::new().set_num_format("@");
        sheet
            .write_number_with_format(3, 0, 123.0_f64, &text_format)
            .expect("write number with Excel text format");

        workbook.save(&path).expect("save workbook");

        match independent_value(&path, 0, 0) {
            Data::Float(value) => assert_eq!(value, 123_456_789_012_345_680.0),
            other => panic!("expected Float, got {other:?}"),
        }
        assert_eq!(
            independent_value(&path, 2, 0),
            Data::String("00123".to_string())
        );
        let xml = ooxml_part(&path, "xl/worksheets/sheet1.xml").expect("worksheet part exists");
        assert!(
            xml.contains("<v>123456789012345680</v>") && xml.contains("<v>0.12345678901234568</v>"),
            "raw numeric serialization changed: {xml}"
        );

        std::fs::remove_file(path).expect("remove temporary workbook");
    }

    #[test]
    fn column_width_freeze_panes_and_merged_ranges_survive_a_write() {
        let path = temp_xlsx("layout");
        let mut workbook = Workbook::new();
        let sheet = workbook.add_worksheet();
        sheet.set_column_width(0, 24.0).expect("set column width");
        sheet.set_freeze_panes(1, 0).expect("set freeze panes");
        sheet
            .merge_range(0, 2, 0, 3, "merged caption", &Format::new())
            .expect("merge range");
        sheet
            .write_string(1, 0, "first body row")
            .expect("write body cell");
        workbook.save(&path).expect("save workbook");

        assert_eq!(
            independent_value(&path, 0, 2),
            Data::String("merged caption".to_string())
        );
        let xml = ooxml_part(&path, "xl/worksheets/sheet1.xml").expect("worksheet part exists");
        assert!(xml.contains("<col "), "column width was not emitted");
        assert!(
            xml.contains("<pane") && xml.contains("state=\"frozen\""),
            "freeze pane was not emitted: {xml}"
        );
        assert!(
            xml.contains("<mergeCell ref=\"C1:D1\""),
            "merged range was not emitted: {xml}"
        );

        std::fs::remove_file(path).expect("remove temporary workbook");
    }

    /// Smallest valid PNG used only to prove that image insertion succeeds.
    fn tiny_png() -> Vec<u8> {
        vec![
            0x89, 0x50, 0x4e, 0x47, 0x0d, 0x0a, 0x1a, 0x0a, 0x00, 0x00, 0x00, 0x0d, 0x49, 0x48,
            0x44, 0x52, 0x00, 0x00, 0x00, 0x20, 0x00, 0x00, 0x00, 0x20, 0x08, 0x02, 0x00, 0x00,
            0x00, 0xfc, 0x18, 0xed, 0xa3, 0x00, 0x00, 0x00, 0x01, 0x73, 0x52, 0x47, 0x42, 0x00,
            0xae, 0xce, 0x1c, 0xe9, 0x00, 0x00, 0x00, 0x04, 0x67, 0x41, 0x4d, 0x41, 0x00, 0x00,
            0xb1, 0x8f, 0x0b, 0xfc, 0x61, 0x05, 0x00, 0x00, 0x00, 0x20, 0x63, 0x48, 0x52, 0x4d,
            0x00, 0x00, 0x7a, 0x26, 0x00, 0x00, 0x80, 0x84, 0x00, 0x00, 0xfa, 0x00, 0x00, 0x00,
            0x80, 0xe8, 0x00, 0x00, 0x75, 0x30, 0x00, 0x00, 0xea, 0x60, 0x00, 0x00, 0x3a, 0x98,
            0x00, 0x00, 0x17, 0x70, 0x9c, 0xba, 0x51, 0x3c, 0x00, 0x00, 0x00, 0x46, 0x49, 0x44,
            0x41, 0x54, 0x48, 0x4b, 0x63, 0xfc, 0xcf, 0x40, 0x63, 0x00, 0xb4, 0x80, 0xa6, 0x88,
            0xb6, 0xa6, 0x83, 0x82, 0x87, 0xa6, 0xce, 0x1f, 0xb5, 0x80, 0x98, 0xe0, 0x1d, 0x8d,
            0x03, 0x82, 0xa1, 0x34, 0x1a, 0x44, 0xa3, 0x41, 0x44, 0x30, 0x04, 0x08, 0x2a, 0x18,
            0x4d, 0x45, 0xa3, 0x41, 0x44, 0x30, 0x04, 0x08, 0x2a, 0x18, 0x4d, 0x45, 0xa3, 0x41,
            0x44, 0x30, 0x04, 0x08, 0x2a, 0x18, 0x4d, 0x45, 0x03, 0x1f, 0x44, 0x00, 0xaa, 0x35,
            0xdd, 0x4e, 0xe6, 0xd5, 0xa1, 0x22, 0x00, 0x00, 0x00, 0x00, 0x49, 0x45, 0x4e, 0x44,
            0xae, 0x42, 0x60, 0x82,
        ]
    }

    #[test]
    fn excel_added_features_can_be_emitted_but_a_values_only_rebuild_drops_all_of_them() {
        let source = temp_xlsx("features-source");
        let rebuilt = temp_xlsx("features-rebuilt");

        let mut workbook = Workbook::new();
        workbook
            .define_name("CaptureTotal", "=Sheet1!$A$2")
            .expect("define name");
        let sheet = workbook.add_worksheet();
        sheet.write_string(0, 0, "heading").expect("write heading");
        sheet.write_number(1, 0, 5.0).expect("write number");
        sheet.write_number(2, 0, 7.0).expect("write number");

        let conditional = ConditionalFormatCell::new()
            .set_rule(ConditionalFormatCellRule::GreaterThan(5))
            .set_format(Format::new().set_background_color(rust_xlsxwriter::Color::Yellow));
        sheet
            .add_conditional_format(1, 0, 2, 0, &conditional)
            .expect("add conditional formatting");
        let validation =
            DataValidation::new().allow_whole_number(DataValidationRule::Between(1, 10));
        sheet
            .add_data_validation(1, 0, 2, 0, &validation)
            .expect("add data validation");
        sheet
            .insert_note(1, 0, &Note::new("reviewer note"))
            .expect("insert note");
        sheet
            .write_url(0, 1, "https://example.invalid/capture")
            .expect("write hyperlink");
        let table = Table::new().set_columns(&[
            TableColumn::new().set_header("heading"),
            TableColumn::new().set_header("link"),
        ]);
        sheet.add_table(0, 0, 1, 1, &table).expect("add table");
        sheet.protect();

        let mut chart = Chart::new(ChartType::Column);
        chart.add_series().set_values("Sheet1!$A$2:$A$3");
        sheet.insert_chart(0, 3, &chart).expect("insert chart");
        sheet
            .insert_image(
                4,
                0,
                &Image::new_from_buffer(&tiny_png()).expect("parse tiny PNG"),
            )
            .expect("insert image");

        workbook.save(&source).expect("save feature workbook");

        let source_names = ooxml_part_names(&source);
        let source_sheet = ooxml_part(&source, "xl/worksheets/sheet1.xml").expect("source sheet");
        let source_workbook = ooxml_part(&source, "xl/workbook.xml").expect("source workbook");
        assert!(
            source_sheet.contains("<conditionalFormatting"),
            "conditional format missing"
        );
        assert!(
            source_sheet.contains("<dataValidation"),
            "data validation missing"
        );
        assert!(
            source_sheet.contains("<sheetProtection"),
            "sheet protection missing"
        );
        assert!(source_sheet.contains("<hyperlink"), "hyperlink missing");
        assert!(
            source_workbook.contains("<definedName"),
            "defined name missing"
        );
        assert!(
            source_names
                .iter()
                .any(|n| n.starts_with("xl/charts/chart")),
            "chart part missing: {source_names:?}"
        );
        assert!(
            source_names.iter().any(|n| n.starts_with("xl/media/")),
            "image part missing: {source_names:?}"
        );
        assert!(
            source_names
                .iter()
                .any(|n| n.starts_with("xl/tables/table")),
            "table part missing: {source_names:?}"
        );
        assert!(
            source_names.iter().any(|n| n.starts_with("xl/comments")),
            "comment/note part missing: {source_names:?}"
        );

        // Simulate the proposed rebuild-from-data flow: read cell values with
        // the independent oracle, then construct a fresh workbook from those
        // values only. Private capture data, not workbook features, is the
        // source of truth in that design.
        let mut oracle: Xlsx<_> = open_workbook(&source).expect("calamine opens source workbook");
        let range = oracle
            .worksheet_range("Sheet1")
            .expect("calamine reads source worksheet");
        let mut rebuilt_workbook = Workbook::new();
        let rebuilt_sheet = rebuilt_workbook.add_worksheet();
        for (row, col, cell) in range.used_cells() {
            match cell {
                Data::String(text) => {
                    rebuilt_sheet
                        .write_string(row as u32, col as u16, text)
                        .expect("rebuild string");
                }
                Data::Float(number) => {
                    rebuilt_sheet
                        .write_number(row as u32, col as u16, *number)
                        .expect("rebuild number");
                }
                Data::Int(number) => {
                    rebuilt_sheet
                        .write_number(row as u32, col as u16, *number as f64)
                        .expect("rebuild integer");
                }
                _ => {}
            }
        }
        rebuilt_workbook
            .save(&rebuilt)
            .expect("save rebuilt workbook");

        let rebuilt_names = ooxml_part_names(&rebuilt);
        let rebuilt_sheet_xml =
            ooxml_part(&rebuilt, "xl/worksheets/sheet1.xml").expect("rebuilt sheet");
        let rebuilt_workbook_xml =
            ooxml_part(&rebuilt, "xl/workbook.xml").expect("rebuilt workbook");
        assert!(!rebuilt_sheet_xml.contains("<conditionalFormatting"));
        assert!(!rebuilt_sheet_xml.contains("<dataValidation"));
        assert!(!rebuilt_sheet_xml.contains("<sheetProtection"));
        assert!(!rebuilt_sheet_xml.contains("<hyperlink"));
        assert!(!rebuilt_workbook_xml.contains("<definedName"));
        assert!(!rebuilt_names
            .iter()
            .any(|n| n.starts_with("xl/charts/chart")));
        assert!(!rebuilt_names.iter().any(|n| n.starts_with("xl/media/")));
        assert!(!rebuilt_names
            .iter()
            .any(|n| n.starts_with("xl/tables/table")));
        assert!(!rebuilt_names.iter().any(|n| n.starts_with("xl/comments")));
        assert_eq!(
            independent_value(&rebuilt, 1, 0),
            Data::Float(5.0),
            "rebuild must keep cell values"
        );

        std::fs::remove_file(source).expect("remove source workbook");
        std::fs::remove_file(rebuilt).expect("remove rebuilt workbook");
    }
}
