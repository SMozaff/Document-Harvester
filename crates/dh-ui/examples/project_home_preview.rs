//! T1 Project Home preview harness.
//!
//! Select a state with PREVIEW_STATE: normal (default), empty, invalid, busy,
//! error, long-path, or minimum. The harness supplies sample presentation data;
//! it performs no filesystem or project operations.

slint::slint! {
    #[edition = "1.19"]
    import { ProjectHomeScreen } from "ui/ProjectHomeScreen.slint";

    export component PreviewWindow inherits Window {
        title: "Document Harvester — Project Home preview";
        preferred-width: 720px;
        preferred-height: 520px;
        min-width: 720px;
        min-height: 520px;

        in property <string> sample-name: "Quarterly report";
        in property <string> sample-parent: "/home/demo/Documents";
        in property <bool> sample-enabled: true;
        in property <string> sample-reason: "";
        in property <bool> sample-busy: false;
        in property <string> sample-error: "";
        in property <string> sample-last-project: "";

        ProjectHomeScreen {
            project-name: root.sample-name;
            parent-directory: root.sample-parent;
            create-enabled: root.sample-enabled;
            create-disabled-reason: root.sample-reason;
            busy: root.sample-busy;
            error-text: root.sample-error;
            last-project-info: root.sample-last-project;
            text-alignment: "left";
            choose-parent-directory => { }
            project-form-changed(name, parent) => { }
            create-project(name, parent) => { }
        }
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let state = std::env::var("PREVIEW_STATE").unwrap_or_else(|_| "normal".to_owned());
    let window = PreviewWindow::new()?;

    match state.as_str() {
        "empty" => {
            window.set_sample_name("".into());
            window.set_sample_parent("".into());
            window.set_sample_enabled(false);
            window.set_sample_reason("Enter a project name.".into());
        }
        "invalid" => {
            window.set_sample_name("Quarterly/report".into());
            window.set_sample_parent("/home/demo/Documents".into());
            window.set_sample_enabled(false);
            window.set_sample_reason("Project names cannot contain '/'.".into());
        }
        "busy" => {
            window.set_sample_name("Quarterly report".into());
            window.set_sample_parent("/home/demo/Documents".into());
            window.set_sample_enabled(false);
            window.set_sample_busy(true);
            window.set_sample_reason("A project is being created.".into());
        }
        "error" => {
            window.set_sample_name("Quarterly report".into());
            window.set_sample_parent("/home/demo/Documents".into());
            window.set_sample_enabled(true);
            window.set_sample_error(
                "The project could not be created. Check the parent directory and try again."
                    .into(),
            );
        }
        "long-path" => {
            window.set_sample_name("Persian quarterly extraction project".into());
            window.set_sample_parent(
                "/home/demo/Documents/2026/Customer archives/Quarterly reports/Source PDFs/Approved"
                    .into(),
            );
            window.set_sample_enabled(true);
            window.set_sample_last_project("Customer archive — restored at launch".into());
        }
        "minimum" | "normal" => {
            window.set_sample_name("Quarterly report".into());
            window.set_sample_parent("/home/demo/Documents".into());
            window.set_sample_enabled(true);
        }
        other => {
            eprintln!(
                "unknown PREVIEW_STATE '{other}'; use normal, empty, invalid, busy, error, long-path, or minimum"
            );
            std::process::exit(2);
        }
    }

    window.run()?;
    Ok(())
}
