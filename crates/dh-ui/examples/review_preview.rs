//! T5 Review preview harness.
//!
//! Select a state with PREVIEW_STATE: ready (default), partial, retry, busy,
//! read-only, or empty. The harness supplies sample presentation data only;
//! callbacks are no-ops and it performs no application or workbook operations.

slint::slint! {
    #[edition = "1.19"]
    import { CapturePreviewPanel } from "ui/ReviewView.slint";

    export component PreviewWindow inherits Window {
        title: "Document Harvester — Review preview";
        preferred-width: 900px;
        preferred-height: 700px;
        min-width: 720px;
        min-height: 520px;

        in property <[string]> sample-table-ids: [];
        in property <[string]> sample-table-labels: [];
        in property <[bool]> sample-table-checked: [];
        in property <[string]> sample-rows: [];
        in property <[string]> sample-warnings: [];
        in property <[string]> sample-unassigned: [];
        in property <string> sample-summary: "";
        in property <string> sample-destination: "";
        in property <string> sample-read-only-note: "";
        in property <int> sample-partial-count: 0;
        in property <bool> sample-partial-required: false;
        in property <bool> sample-partial-confirmed: false;
        in property <bool> sample-approve-enabled: false;
        in property <bool> sample-cancel-enabled: true;
        in property <string> sample-commit-label: "Approve & commit";

        CapturePreviewPanel {
            preview-table-ids: root.sample-table-ids;
            preview-table-labels: root.sample-table-labels;
            preview-table-checked: root.sample-table-checked;
            preview-rows: root.sample-rows;
            preview-warnings: root.sample-warnings;
            preview-unassigned: root.sample-unassigned;
            preview-summary: root.sample-summary;
            destination-label: root.sample-destination;
            read-only-note: root.sample-read-only-note;
            partial-cell-count: root.sample-partial-count;
            partial-confirmation-required: root.sample-partial-required;
            partial-confirmed: root.sample-partial-confirmed;
            approve-enabled: root.sample-approve-enabled;
            cancel-enabled: root.sample-cancel-enabled;
            commit-label: root.sample-commit-label;
            table-selection-changed(table_id, checked) => { }
            confirm-partial-cells => { }
            approve-preview => { }
            cancel-preview => { }
        }
    }
}

fn string_model(values: &[&str]) -> slint::ModelRc<slint::SharedString> {
    let values = values
        .iter()
        .map(|value| slint::SharedString::from(*value))
        .collect::<Vec<_>>();
    slint::ModelRc::from(values.as_slice())
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let state = std::env::var("PREVIEW_STATE").unwrap_or_else(|_| "ready".to_owned());
    let window = PreviewWindow::new()?;

    window.set_sample_table_ids(string_model(&["table-id-01", "table-id-02"]));
    window.set_sample_table_labels(string_model(&[
        "Table 1 · 4 rows · 3 columns",
        "Table 2 · 2 rows · 2 columns",
    ]));
    window.set_sample_table_checked(slint::ModelRc::from([true, false].as_slice()));
    window.set_sample_rows(string_model(&[
        "Table 1 · row 1 | North │ 14 │ 2026-01-15",
        "Table 1 · row 2 | South │ 18 │ 2026-01-16",
        "Table 1 · row 3 | East │ 11 │ 2026-01-17",
        "Table 1 · row 4 | West │ 9 │ 2026-01-18",
    ]));
    window.set_sample_warnings(string_model(&[
        "A partial cell crosses the selected region boundary.",
    ]));
    window.set_sample_unassigned(string_model(&[
        "Page 2 · (42.00, 30.00)–(88.00, 44.00) · unassigned · Note",
    ]));
    window.set_sample_summary(
        "Project sample-project · PDF /sample/reports/quarterly.pdf · page 2 · region (18.00, 22.00)–(184.00, 126.00)".into(),
    );
    window.set_sample_destination("Rows 12–17 in the project workbook".into());
    window.set_sample_partial_count(1);
    window.set_sample_partial_required(true);
    window.set_sample_approve_enabled(false);

    match state.as_str() {
        "ready" => window.set_sample_approve_enabled(true),
        "partial" => {
            window.set_sample_partial_required(false);
            window.set_sample_partial_confirmed(true);
            window.set_sample_approve_enabled(true);
        }
        "retry" => {
            window.set_sample_partial_required(false);
            window.set_sample_partial_confirmed(true);
            window.set_sample_approve_enabled(true);
            window.set_sample_commit_label("Retry commit".into());
        }
        "busy" => {
            window.set_sample_partial_required(false);
            window.set_sample_partial_confirmed(true);
            window.set_sample_cancel_enabled(false);
            window.set_sample_commit_label("Committing…".into());
        }
        "read-only" => {
            window.set_sample_read_only_note(
                "The project is read-only. Commit is unavailable until the application resolves the workbook state.".into(),
            );
        }
        "empty" => {
            window.set_sample_table_ids(string_model(&[]));
            window.set_sample_table_labels(string_model(&[]));
            window.set_sample_table_checked(slint::ModelRc::from([].as_slice()));
            window.set_sample_rows(string_model(&[]));
            window.set_sample_warnings(string_model(&[]));
            window.set_sample_unassigned(string_model(&[]));
        }
        other => {
            eprintln!(
                "unknown PREVIEW_STATE '{other}'; use ready, partial, retry, busy, read-only, or empty"
            );
            std::process::exit(2);
        }
    }

    window.run()?;
    Ok(())
}
