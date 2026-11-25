mod navigation;
mod preview;
mod viewer;

use std::env;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};

use dh_ir::{BBox, PageTextExtractor};
use dh_part1::renderer::{PageInfo, PageRenderer, PdfiumRenderer, RenderedPage};
use dh_part3::{PageTransform, RegionSelection};
use slint::{ComponentHandle, Image, Rgba8Pixel, SharedPixelBuffer, SharedString};

use navigation::{
    availability, empty_state_message, Action, Availability, Navigation, NavigationEvent,
    ProjectForm, Screen, WorkspaceAccess,
};
use preview::{CapturePreview, PreviewIdentity, PreviewWorkflow};
use viewer::ViewerState;

#[derive(Debug, Default, PartialEq, Eq)]
struct CliOptions {
    part: Option<u8>,
    input: Option<PathBuf>,
    output: Option<PathBuf>,
    column_order: Option<String>,
    triage_only: bool,
    force: bool,
}

fn usage() -> &'static str {
    "usage: document-harvester [--part 1|2|3] [--input PATH] [--output PATH] \\\n       --column-order right_to_left|left_to_right [--triage-only] [--force]\n       document-harvester --pdfium-self-test"
}

fn parse(args: &[String]) -> Result<CliOptions, String> {
    let mut options = CliOptions::default();
    let mut index = 0;
    while index < args.len() {
        let key = args[index].as_str();
        match key {
            "--triage-only" => options.triage_only = true,
            "--force" => options.force = true,
            "--part" | "--input" | "--output" | "--column-order" => {
                index += 1;
                let value = args
                    .get(index)
                    .ok_or_else(|| format!("missing value for {key}"))?;
                match key {
                    "--part" => options.part = Some(value.parse().map_err(|_| "invalid part")?),
                    "--input" => options.input = Some(PathBuf::from(value)),
                    "--output" => options.output = Some(PathBuf::from(value)),
                    "--column-order" => options.column_order = Some(value.clone()),
                    _ => unreachable!(),
                }
            }
            other => return Err(format!("unknown option: {other}")),
        }
        index += 1;
    }
    if let Some(part) = options.part {
        if !(1..=3).contains(&part) {
            return Err("part must be 1, 2 or 3".to_owned());
        }
    }
    Ok(options)
}

/// Push the current navigation state into the shell.
fn apply_screen(window: &dh_ui::MainWindow, navigation: Navigation) {
    window.set_project_open(navigation.project_open());
    window.set_empty_state_message(empty_state_message(navigation.screen()).into());
}

/// Push the Create Project availability, and its explanation when disabled,
/// into the shell. The form only exists on project home.
fn refresh_create_project(
    window: &dh_ui::MainWindow,
    name: &str,
    parent_directory: &str,
    busy: bool,
) {
    match availability(
        Screen::ProjectHome,
        WorkspaceAccess::Writable,
        Action::CreateProject,
        ProjectForm::for_values(name, parent_directory, busy),
    ) {
        Availability::Enabled => {
            window.set_create_project_enabled(true);
            window.set_create_project_note("".into());
        }
        Availability::Disabled(reason) => {
            window.set_create_project_enabled(false);
            window.set_create_project_note(reason.into());
        }
        Availability::Hidden => {
            window.set_create_project_enabled(false);
            window.set_create_project_note("".into());
        }
    }
}

/// Push the PDF-import availability for the workspace access.
///
/// A read-only project explains why import is disabled even when the bundled
/// renderer is available; an unavailable renderer keeps its own explanation.
fn refresh_import_availability(
    window: &dh_ui::MainWindow,
    access: WorkspaceAccess,
    pdfium_error: Option<&str>,
) {
    match availability(
        Screen::ProjectWorkspace,
        access,
        Action::ImportPdf,
        ProjectForm::default(),
    ) {
        Availability::Enabled => match pdfium_error {
            Some(error) => {
                window.set_import_pdf_enabled(false);
                window.set_import_pdf_note(error.into());
            }
            None => {
                window.set_import_pdf_enabled(true);
                window.set_import_pdf_note(navigation::IMPORT_PDF_NOTE.into());
            }
        },
        Availability::Disabled(reason) => {
            window.set_import_pdf_enabled(false);
            window.set_import_pdf_note(reason.into());
        }
        Availability::Hidden => {
            window.set_import_pdf_enabled(false);
            window.set_import_pdf_note("".into());
        }
    }
}

/// How the launch-restore path found the private project state.
enum StoredProjectOpen {
    NoProject,
    WorkbookMissing,
    Opened(Box<dh_project::ProjectOpenOutcome>),
}

/// Route launch restore through the single recovery-aware open entry point.
fn open_stored_project(store: &dh_project::ProjectStore) -> dh_core::Result<StoredProjectOpen> {
    match store.current_project()? {
        None => Ok(StoredProjectOpen::NoProject),
        Some(project) if !project.workbook_path.is_file() => Ok(StoredProjectOpen::WorkbookMissing),
        Some(project) => store
            .open_project(&project.id)
            .map(|outcome| StoredProjectOpen::Opened(Box::new(outcome))),
    }
}

/// Create a project, then open it through the same recovery-aware entry point.
///
/// The workspace is never shown before recovery has run, including for a newly
/// created project.
fn create_and_open_project(
    store: &dh_project::ProjectStore,
    name: &str,
    parent_directory: &std::path::Path,
) -> dh_core::Result<dh_project::ProjectOpenOutcome> {
    let project = store.create_project(name, parent_directory, &dh_writer::XlsxWriter)?;
    store.open_project(&project.id)
}

type RegionSelectionStore = Arc<Mutex<Option<BBox>>>;
type PreviewWorkflowStore = Arc<Mutex<PreviewWorkflow>>;
type RegionDragStore = Arc<Mutex<Option<RegionDrag>>>;

/// The two pieces of viewer state the project-open path resets together.
struct ViewerContext<'a> {
    state: &'a Arc<Mutex<ViewerState>>,
    selection: &'a RegionSelectionStore,
}

#[derive(Debug, Clone, PartialEq)]
struct RegionDrag {
    kind: String,
    start_x_px: f64,
    start_y_px: f64,
    base_rect_px: (f64, f64, f64, f64),
}

/// Build the canonical transform for the currently displayed page.
///
/// The arithmetic lives in `PageTransform` and `viewer::layout`; this only
/// supplies the current viewer state, including the pan-adjusted origin.
fn current_page_transform(
    state: &ViewerState,
) -> dh_core::Result<(PageTransform, viewer::PageLayout)> {
    let info = state.info.clone().ok_or_else(|| {
        dh_core::DhError::InvalidInput("no page is rendered for region selection".to_owned())
    })?;
    let layout = viewer::layout(
        &info,
        state.zoom,
        (state.pan_x_logical, state.pan_y_logical),
        (state.viewport_width_logical, state.viewport_height_logical),
        state.device_scale,
    )?;
    Ok((
        PageTransform {
            page_box: info.page_box,
            rotation_degrees: info.rotation_degrees,
            zoom: state.zoom,
            device_scale: state.device_scale,
            origin_x_px: layout.image_x_logical * state.device_scale,
            origin_y_px: layout.image_y_logical * state.device_scale,
        },
        layout,
    ))
}

fn page_rect_from_screen_px(
    state: &ViewerState,
    rectangle_px: (f64, f64, f64, f64),
) -> dh_core::Result<BBox> {
    let (transform, _) = current_page_transform(state)?;
    RegionSelection {
        page: state.page_number,
        rectangle: BBox {
            x0: rectangle_px.0,
            y0: rectangle_px.1,
            x1: rectangle_px.2,
            y1: rectangle_px.3,
        },
    }
    .to_page_user_space(&transform)
}

/// Project an accepted page-space rectangle back to overlay-local logical px.
///
/// `page_rect_to_screen` is the canonical projection and returns physical
/// pixels; dividing once by the transform's device scale yields the logical
/// coordinates used by Slint. No placement arithmetic is repeated here.
fn logical_rect_from_page_rect(
    transform: &PageTransform,
    page_rect: BBox,
) -> dh_core::Result<(f64, f64, f64, f64)> {
    let screen = transform.page_rect_to_screen(page_rect)?;
    Ok((
        screen.x0 / transform.device_scale,
        screen.y0 / transform.device_scale,
        screen.x1 / transform.device_scale,
        screen.y1 / transform.device_scale,
    ))
}

fn candidate_screen_rect_px(
    drag: &RegionDrag,
    current_x_px: f64,
    current_y_px: f64,
) -> (f64, f64, f64, f64) {
    let dx = current_x_px - drag.start_x_px;
    let dy = current_y_px - drag.start_y_px;
    let (x0, y0, x1, y1) = drag.base_rect_px;
    match drag.kind.as_str() {
        "draw" => (
            drag.start_x_px.min(current_x_px),
            drag.start_y_px.min(current_y_px),
            drag.start_x_px.max(current_x_px),
            drag.start_y_px.max(current_y_px),
        ),
        "move" => (x0 + dx, y0 + dy, x1 + dx, y1 + dy),
        "left" => (x0 + dx, y0, x1, y1),
        "top" => (x0, y0 + dy, x1, y1),
        "top-left" => (x0 + dx, y0 + dy, x1, y1),
        "right" => (x0, y0, x1 + dx, y1),
        "top-right" => (x0, y0 + dy, x1 + dx, y1),
        "bottom" => (x0, y0, x1, y1 + dy),
        "bottom-right" => (x0, y0, x1 + dx, y1 + dy),
        "bottom-left" => (x0 + dx, y0, x1, y1 + dy),
        _ => drag.base_rect_px,
    }
}

fn page_geometry_text(state: &ViewerState) -> String {
    let Some(info) = state.info.as_ref() else {
        return "Page geometry unavailable".to_owned();
    };
    format!(
        "Page box ({:.2}, {:.2})–({:.2}, {:.2}) pt, size {:.2} × {:.2} pt, rotation {}°, zoom {:.2}×",
        info.page_box.x0,
        info.page_box.y0,
        info.page_box.x1,
        info.page_box.y1,
        info.page_box.x1 - info.page_box.x0,
        info.page_box.y1 - info.page_box.y0,
        info.rotation_degrees,
        state.zoom,
    )
}

fn region_page_text(page_rect: Option<BBox>) -> String {
    match page_rect {
        Some(rect) => format!(
            "Selection in page coordinates: ({:.2}, {:.2})–({:.2}, {:.2}) pt",
            rect.x0, rect.y0, rect.x1, rect.y1
        ),
        None => "No region selected".to_owned(),
    }
}

fn current_preview_identity(
    store: &dh_project::ProjectStore,
    state: &ViewerState,
    region: BBox,
) -> dh_core::Result<PreviewIdentity> {
    let project = store
        .current_project()?
        .ok_or_else(|| dh_core::DhError::State("no active project for preview".to_owned()))?;
    if project.id != state.project_id {
        return Err(dh_core::DhError::State(
            "the active project changed after the preview was created".to_owned(),
        ));
    }
    let pdf_path = state
        .pdf_path
        .as_ref()
        .ok_or_else(|| dh_core::DhError::State("no imported PDF is available".to_owned()))?;
    if project.imported_pdf_path.as_ref() != Some(pdf_path) {
        return Err(dh_core::DhError::State(
            "the imported PDF changed after the preview was created".to_owned(),
        ));
    }
    let source_pdf_sha256 = dh_writer::file_sha256(pdf_path)?;
    let workbook_sha256 = dh_writer::file_sha256(&project.workbook_path)?;
    Ok(PreviewIdentity {
        project_id: project.id,
        source_pdf_path: pdf_path.to_string_lossy().to_string(),
        source_pdf_sha256,
        workbook_path: project.workbook_path.to_string_lossy().to_string(),
        workbook_sha256,
        page: state.page_number,
        region,
        policy_version: preview::PREVIEW_POLICY_VERSION.to_owned(),
    })
}

/// Build a review-only preview using the Stage 3 page reconstruction and
/// region filter. No workbook-writing API is called here.
fn generate_capture_preview(
    store: &dh_project::ProjectStore,
    state: &ViewerState,
    region: BBox,
) -> dh_core::Result<CapturePreview> {
    let identity = current_preview_identity(store, state, region)?;
    let pdf_path = state
        .pdf_path
        .as_ref()
        .ok_or_else(|| dh_core::DhError::State("no imported PDF is available".to_owned()))?;
    let (transform, _layout) = current_page_transform(state)?;
    let screen_rectangle = transform.page_rect_to_screen(region)?;
    let selection = RegionSelection {
        page: state.page_number,
        rectangle: screen_rectangle,
    };

    let extractor = dh_part1::text::PdfiumTextExtractor::load_bundled()?;
    let page = extractor.extract_page(pdf_path, state.page_number)?;
    if let Some(reason) = page.unsupported_reason.as_deref() {
        return Err(dh_core::DhError::Parse(format!(
            "page {} has no usable digital text layer: {reason}; OCR is not used",
            state.page_number
        )));
    }
    let options = dh_core::grid::ReconstructionOptions {
        extractor_name: "pdfium-digital-v1".to_owned(),
        extractor_version: preview::DIGITAL_EXTRACTOR_VERSION.to_owned(),
        numeric_policy_version: dh_core::grid::NUMERIC_POLICY_VERSION.to_owned(),
        ..dh_core::grid::ReconstructionOptions::default()
    };
    let reconstructed =
        dh_core::grid::reconstruct_page(state.page_number, &page.runs, &options)?;
    let filtered = dh_part3::extraction::filter_reconstructed_region(
        &page,
        &reconstructed,
        &transform,
        &selection,
    )?;
    if filtered.tables.is_empty() && !filtered.unassigned_text.is_empty() {
        return Err(dh_core::DhError::State(
            "preview cannot attribute unassigned text because the selected region contains no reconstructed table"
                .to_owned(),
        ));
    }
    let existing_captures = store.capture_list(&identity.project_id)?;
    Ok(CapturePreview::from_region_result(
        identity,
        filtered,
        &existing_captures,
    ))
}

fn refresh_preview_ui(
    window: &dh_ui::MainWindow,
    workflow: &PreviewWorkflowStore,
) {
    let workflow = workflow.lock().expect("preview workflow lock");
    let writable = window.get_workspace_writable();
    let Some(preview) = workflow.pending.as_ref().filter(|_| workflow.preview_shown) else {
        window.set_preview_visible(false);
        window.set_preview_rows(Vec::<SharedString>::new().into());
        window.set_preview_table_ids(Vec::<SharedString>::new().into());
        window.set_preview_table_labels(Vec::<SharedString>::new().into());
        window.set_preview_table_checked(Vec::<bool>::new().into());
        window.set_preview_selection_enabled(true);
        window.set_preview_warnings(Vec::<SharedString>::new().into());
        window.set_preview_unassigned(Vec::<SharedString>::new().into());
        window.set_preview_summary("".into());
        window.set_preview_destination("".into());
        window.set_preview_partial_count(0);
        window.set_preview_partial_confirmation_required(false);
        window.set_preview_partial_confirmed(false);
        window.set_preview_commit_enabled(false);
        window.set_preview_cancel_enabled(true);
        window.set_preview_commit_label("Approve & commit".into());
        window.set_preview_read_only_note("".into());
        return;
    };

    window.set_preview_visible(true);
    window.set_preview_table_ids(
        preview.tables.iter().map(|table| SharedString::from(table.id.as_str())).collect::<Vec<_>>().into(),
    );
    window.set_preview_table_labels(
        preview.tables.iter().map(|table| SharedString::from(table.label.as_str())).collect::<Vec<_>>().into(),
    );
    window.set_preview_table_checked(
        preview.tables.iter().map(|table| preview.selected_table_ids.contains(&table.id)).collect::<Vec<_>>().into(),
    );
    window.set_preview_selection_enabled(!workflow.commit_in_progress);
    window.set_preview_rows(
        preview
            .rows
            .iter()
            .map(|value| SharedString::from(value.as_str()))
            .collect::<Vec<_>>()
            .into(),
    );
    window.set_preview_warnings(
        preview
            .warnings
            .iter()
            .map(|value| SharedString::from(value.as_str()))
            .collect::<Vec<_>>()
            .into(),
    );
    window.set_preview_unassigned(
        preview
            .unassigned
            .iter()
            .map(|value| SharedString::from(value.as_str()))
            .collect::<Vec<_>>()
            .into(),
    );
    window.set_preview_summary(
        format!(
            "Project {} · PDF {} · page {} · region ({:.2}, {:.2})–({:.2}, {:.2})",
            preview.identity.project_id,
            preview.identity.source_pdf_path,
            preview.identity.page,
            preview.identity.region.x0,
            preview.identity.region.y0,
            preview.identity.region.x1,
            preview.identity.region.y1
        )
        .into(),
    );
    window.set_preview_destination(preview.destination_label().into());
    window.set_preview_partial_count(
        i32::try_from(preview.partial_cell_count).unwrap_or(i32::MAX),
    );
    window.set_preview_partial_confirmation_required(
        preview.partial_cell_count > 0 && !workflow.partial_cells_confirmed,
    );
    window.set_preview_partial_confirmed(workflow.partial_cells_confirmed);
    window.set_preview_cancel_enabled(!workflow.commit_in_progress);
    window.set_preview_commit_enabled(
        !workflow.commit_in_progress
            && (workflow.can_approve(writable) || workflow.commit_enabled(writable)),
    );
    window.set_preview_commit_label(
        if workflow.commit_in_progress {
            "Committing…"
        } else if workflow.commit_approved {
            "Retry commit"
        } else {
            "Approve & commit"
        }
        .into(),
    );
    window.set_preview_read_only_note(
        if writable {
            ""
        } else {
            navigation::READ_ONLY_NOTE
        }
        .into(),
    );
}

fn start_preview_generation(
    window: &dh_ui::MainWindow,
    store: dh_project::ProjectStore,
    state: Arc<Mutex<ViewerState>>,
    selection: RegionSelectionStore,
    workflow: PreviewWorkflowStore,
) {
    let Some(region) = *selection.lock().expect("region selection lock") else {
        window.set_status_message(
            "Select a valid region on the current PDF page before generating a preview.".into(),
        );
        return;
    };
    let snapshot = state.lock().expect("viewer state lock").clone();
    {
        workflow.lock().expect("preview workflow lock").clear();
    }
    refresh_preview_ui(window, &workflow);
    window.set_viewer_busy(true);
    window.set_viewer_status("Extracting digital text and building capture preview…".into());

    let weak_window = window.as_weak();
    std::thread::spawn(move || {
        let result = generate_capture_preview(&store, &snapshot, region);
        let _ = weak_window.upgrade_in_event_loop(move |window| {
            window.set_viewer_busy(false);
            match result {
                Ok(preview) => {
                    workflow
                        .lock()
                        .expect("preview workflow lock")
                        .show(preview);
                    refresh_preview_ui(&window, &workflow);
                    window.set_viewer_status(
                        "Preview ready. The workbook has not been modified.".into(),
                    );
                    window.set_status_message(
                        "Review the capture preview and explicitly approve it to commit.".into(),
                    );
                }
                Err(error) => {
                    refresh_preview_ui(&window, &workflow);
                    window.set_viewer_status(format!("Preview failed: {error}").into());
                    window.set_status_message(
                        "No workbook changes were made; correct the selection or source and retry."
                            .into(),
                    );
                }
            }
        });
    });
}

/// The approved capture is rebuilt with the existing journalled writer path;
/// private project state is updated only after the verified replacement.
fn commit_capture_preview(
    store: &dh_project::ProjectStore,
    preview: &CapturePreview,
    viewer_state: &ViewerState,
) -> dh_core::Result<bool> {
    if preview.selected_table_ids.is_empty() || preview.capture.blocks.is_empty() {
        return Err(dh_core::DhError::State(
            "commit refused: select at least one reconstructed table".to_owned(),
        ));
    }
    if !preview.has_valid_approval() {
        return Err(dh_core::DhError::State(
            "commit refused: the checked table set has no matching explicit approval".to_owned(),
        ));
    }
    store.recover_pending_transaction(&preview.identity.project_id)?;
    let current_identity = current_preview_identity(store, viewer_state, preview.identity.region)?;
    if preview.identity.differs_from(&current_identity) {
        return Err(dh_core::DhError::State(
            "stale preview refused: the project, source PDF, page, region or workbook changed before commit"
                .to_owned(),
        ));
    }
    let project = store
        .current_project()?
        .ok_or_else(|| dh_core::DhError::State("no active project for commit".to_owned()))?;
    if project.id != preview.identity.project_id
        || project.workbook_path.to_string_lossy().as_ref() != preview.identity.workbook_path.as_str()
    {
        return Err(dh_core::DhError::State(
            "the project or destination workbook changed after the preview".to_owned(),
        ));
    }

    let mut captures = store.capture_list(&project.id)?;
    // A retry after journal recovery must be idempotent: an already adopted
    // capture is not appended for a second time.
    if !preview::append_capture_once(&mut captures, &preview.capture) {
        return Ok(false);
    }
    let committed_hash = store
        .committed_workbook_sha256(&project.id)?
        .unwrap_or_else(|| preview.identity.workbook_sha256.clone());
    let journal = dh_writer::TransactionJournal::new(&store.journal_path(&project.id));
    let workbook_sha256 = dh_writer::XlsxWriter.write_capture_list_with_journal(
        &captures,
        &project.workbook_path,
        &committed_hash,
        &journal,
        None,
        None,
    )?;
    store.commit_capture_list(&project.id, &captures, workbook_sha256)?;
    Ok(true)
}


/// Read capture history and determine whether latest-capture undo is safe.
fn capture_history_projection(
    store: &dh_project::ProjectStore,
    project_id: &str,
) -> dh_core::Result<(Vec<String>, bool, String)> {
    let project = store
        .current_project()?
        .filter(|project| project.id == project_id)
        .ok_or_else(|| dh_core::DhError::State("the active project changed".to_owned()))?;
    let history = store.capture_history(project_id)?;
    let rows = history.iter().enumerate().map(|(index, entry)| {
        format!(
            "Capture {} · page {} · rows {}–{} · region ({:.2}, {:.2})–({:.2}, {:.2})",
            index + 1,
            entry.capture.page,
            entry.destination_start_row + 1,
            entry.destination_end_row,
            entry.capture.region.x0,
            entry.capture.region.y0,
            entry.capture.region.x1,
            entry.capture.region.y1
        )
    }).collect::<Vec<_>>();
    if history.is_empty() {
        return Ok((rows, false, "No committed captures to undo.".to_owned()));
    }
    let expected = store.committed_workbook_sha256(project_id)?;
    let actual = dh_writer::file_sha256(&project.workbook_path)?;
    if expected.as_deref() != Some(actual.as_str()) {
        return Ok((rows, false,
            "Undo disabled: the workbook differs from the last committed version; undo is unsafe.".to_owned()));
    }
    Ok((rows, true, "Undo removes only the latest committed capture.".to_owned()))
}

/// Load history off the UI thread; Slint receives only display data.
fn refresh_history_async(
    window: &dh_ui::MainWindow,
    store: dh_project::ProjectStore,
    project_id: String,
    writable: bool,
) {
    window.set_history_busy(true);
    window.set_undo_enabled(false);
    window.set_undo_note(
        if writable { "Checking capture history and workbook integrity…" }
        else { navigation::READ_ONLY_NOTE }.into(),
    );
    let weak_window = window.as_weak();
    std::thread::spawn(move || {
        let result = capture_history_projection(&store, &project_id);
        let _ = weak_window.upgrade_in_event_loop(move |window| {
            window.set_history_busy(false);
            match result {
                Ok((rows, safe, note)) => {
                    window.set_history_rows(
                        rows.iter().map(|row| SharedString::from(row.as_str()))
                            .collect::<Vec<_>>().into(),
                    );
                    window.set_undo_enabled(writable && safe);
                    window.set_undo_note(
                        if writable { note } else { navigation::READ_ONLY_NOTE.to_owned() }.into(),
                    );
                    window.set_history_empty_message(
                        if rows.is_empty() { "No captures have been committed yet." } else { "" }.into(),
                    );
                }
                Err(error) => {
                    window.set_history_rows(Vec::<SharedString>::new().into());
                    window.set_history_empty_message("Capture history is unavailable.".into());
                    window.set_undo_enabled(false);
                    window.set_undo_note(format!("Undo disabled: {error}").into());
                }
            }
        });
    });
}

fn clear_region_selection(window: &dh_ui::MainWindow, selection: &RegionSelectionStore) {
    *selection.lock().expect("region selection lock") = None;
    window.set_region_visible(false);
    window.set_region_page_text(region_page_text(None).into());
}

fn refresh_region_overlay(
    window: &dh_ui::MainWindow,
    state: &ViewerState,
    selection: &RegionSelectionStore,
) {
    window.set_page_geometry_text(page_geometry_text(state).into());
    let page_rect = *selection.lock().expect("region selection lock");
    window.set_region_page_text(region_page_text(page_rect).into());
    let Some(page_rect) = page_rect else {
        window.set_region_visible(false);
        return;
    };
    match current_page_transform(state)
        .and_then(|(transform, _)| logical_rect_from_page_rect(&transform, page_rect))
    {
        Ok((x0, y0, x1, y1)) => {
            window.set_region_x(x0 as f32);
            window.set_region_y(y0 as f32);
            window.set_region_width((x1 - x0) as f32);
            window.set_region_height((y1 - y0) as f32);
            window.set_region_visible(true);
        }
        Err(error) => {
            window.set_region_visible(false);
            window.set_viewer_status(format!("Region projection failed: {error}").into());
        }
    }
}

fn apply_region_candidate(
    window: &dh_ui::MainWindow,
    state: &ViewerState,
    selection: &RegionSelectionStore,
    rectangle_px: (f64, f64, f64, f64),
) {
    match page_rect_from_screen_px(state, rectangle_px) {
        Ok(page_rect) => {
            *selection.lock().expect("region selection lock") = Some(page_rect);
            refresh_region_overlay(window, state, selection);
            window.set_viewer_status(
                format!("Region accepted: {}", region_page_text(Some(page_rect))).into(),
            );
        }
        Err(error) => {
            // Keep the last accepted selection; never clip an invalid rectangle.
            window.set_viewer_status(format!("Region rejected: {error}").into());
            refresh_region_overlay(window, state, selection);
        }
    }
}

/// Show a project returned by the recovery-aware open entry point.
///
/// A read-only outcome still shows the workspace, with commit, undo and import
/// disabled and the plain recovery reason in the status line.
fn present_open_project(
    window: &dh_ui::MainWindow,
    navigation: &mut Navigation,
    viewer: &ViewerContext<'_>,
    outcome: &dh_project::ProjectOpenOutcome,
    renderer: Option<&Arc<dyn PageRenderer>>,
    pdfium_error: Option<&str>,
    restored: bool,
    store: &dh_project::ProjectStore,
) {
    let project = outcome.project();
    window.set_project_name(project.name.clone().into());
    window.set_parent_directory(
        project
            .project_directory
            .parent()
            .unwrap_or(&project.project_directory)
            .to_string_lossy()
            .to_string()
            .into(),
    );
    window.set_workbook_path(project.workbook_path.to_string_lossy().to_string().into());

    let imported_copy = project
        .imported_pdf_path
        .as_ref()
        .filter(|path| path.is_file())
        .cloned();
    {
        let mut state = viewer.state.lock().expect("viewer state lock");
        *state = ViewerState::new();
        state.project_id = project.id.clone();
        state.pdf_path = imported_copy.clone();
    }
    clear_region_selection(window, viewer.selection);
    window.set_pdf_imported(imported_copy.is_some());
    if project.imported_pdf_path.is_some() && imported_copy.is_none() {
        window.set_viewer_status("The imported PDF copy is missing; import the PDF again.".into());
    }

    let read_only_reason = outcome.read_only_reason().map(str::to_string);
    let writable = read_only_reason.is_none();
    let access = if read_only_reason.is_some() {
        WorkspaceAccess::ReadOnly
    } else {
        WorkspaceAccess::Writable
    };
    navigation.apply(if access == WorkspaceAccess::ReadOnly {
        NavigationEvent::ProjectOpenedReadOnly
    } else {
        NavigationEvent::ProjectOpened
    });
    debug_assert_eq!(navigation.workspace_access(), access);
    refresh_import_availability(window, access, pdfium_error);
    window.set_workspace_writable(read_only_reason.is_none());

    match read_only_reason {
        Some(reason) => {
            window.set_status_message(format!("Project opened read-only: {reason}").into());
        }
        None => {
            window.set_status_message(
                if restored {
                    "Last project restored from private application data."
                } else {
                    "Project and blank workbook created."
                }
                .into(),
            );
        }
    }
    apply_screen(window, *navigation);
    refresh_history_async(window, store.clone(), project.id.clone(), writable);

    if window.get_pdf_imported() {
        if let Some(renderer) = renderer {
            start_render(window, viewer.state, renderer, viewer.selection, true);
        }
    }
}

/// Plain data returned by a worker thread. Slint objects never cross threads.
struct RenderOutcome {
    generation: u64,
    page_number: u32,
    zoom: f64,
    info: PageInfo,
    layout: viewer::PageLayout,
    rendered: RenderedPage,
}

/// Start one render on a worker thread and publish plain pixels on the UI thread.
///
/// The PDF path comes from the private app-data copy held in `state`; the
/// original source path is never used here.
fn start_render(
    window: &dh_ui::MainWindow,
    state: &Arc<Mutex<ViewerState>>,
    renderer: &Arc<dyn PageRenderer>,
    region_selection: &RegionSelectionStore,
    fit: bool,
) {
    let weak_window = window.as_weak();
    let (pdf_path, page_number, zoom, pan, generation, device_scale, viewport) = {
        let mut state = state.lock().expect("viewer state lock");
        state.generation += 1;
        (
            state.pdf_path.clone(),
            state.page_number,
            state.zoom,
            (state.pan_x_logical, state.pan_y_logical),
            state.generation,
            f64::from(window.window().scale_factor()),
            (
                f64::from(window.get_viewport_width()),
                f64::from(window.get_viewport_height()),
            ),
        )
    };
    let Some(pdf_path) = pdf_path else {
        return;
    };
    // A freshly created window may not have run layout yet; a modest fallback
    // only affects this first fit and is replaced by the real viewport later.
    let viewport = if viewport.0 <= 0.0 || viewport.1 <= 0.0 {
        (600.0, 400.0)
    } else {
        viewport
    };

    window.set_viewer_busy(true);
    window.set_viewer_status("Rendering page…".into());
    let state_for_worker = state.clone();
    let region_selection_for_worker = region_selection.clone();
    let renderer_for_worker = renderer.clone();
    std::thread::spawn(move || {
        let result = (|| -> dh_core::Result<RenderOutcome> {
            let info = renderer_for_worker.page_info(&pdf_path, page_number)?;
            let zoom = if fit {
                viewer::fit_zoom(&info, viewport.0, viewport.1, device_scale)?
            } else {
                zoom
            };
            let layout = viewer::layout(&info, zoom, pan, viewport, device_scale)?;
            let rendered = renderer_for_worker.render_page(
                &pdf_path,
                page_number,
                layout.render_width_px,
                layout.render_height_px,
            )?;
            Ok(RenderOutcome {
                generation,
                page_number,
                zoom,
                info,
                layout,
                rendered,
            })
        })();

        let _ = weak_window.upgrade_in_event_loop(move |window| match result {
            Ok(outcome) => {
                let mut state = state_for_worker.lock().expect("viewer state lock");
                if state.generation != outcome.generation {
                    return;
                }
                state.info = Some(outcome.info.clone());
                state.page_count = outcome.info.page_count;
                state.page_number = outcome.page_number;
                state.zoom = outcome.zoom;
                state.device_scale = device_scale;
                state.viewport_width_logical = viewport.0;
                state.viewport_height_logical = viewport.1;
                let refreshed_state = state.clone();
                drop(state);

                let buffer = SharedPixelBuffer::<Rgba8Pixel>::clone_from_slice(
                    &outcome.rendered.pixels_rgba,
                    outcome.rendered.width_px,
                    outcome.rendered.height_px,
                );
                window.set_page_image(Image::from_rgba8(buffer));
                window.set_image_x(outcome.layout.image_x_logical as f32);
                window.set_image_y(outcome.layout.image_y_logical as f32);
                window.set_image_width(outcome.layout.image_width_logical as f32);
                window.set_image_height(outcome.layout.image_height_logical as f32);
                window.set_page_number(i32::try_from(outcome.page_number).unwrap_or(i32::MAX));
                window.set_page_count(i32::try_from(outcome.info.page_count).unwrap_or(i32::MAX));
                window.set_viewer_status(
                    format!(
                        "Zoom {:.2}× — {} × {} render pixels",
                        outcome.zoom, outcome.rendered.width_px, outcome.rendered.height_px
                    )
                    .into(),
                );
                refresh_region_overlay(&window, &refreshed_state, &region_selection_for_worker);
                window.set_viewer_busy(false);
            }
            Err(error) => {
                window.set_viewer_status(format!("Render failed: {error}").into());
                window.set_viewer_busy(false);
            }
        });
    });
}

/// Reposition the already rendered image after a pan, using PageTransform only.
fn refresh_pan(
    window: &dh_ui::MainWindow,
    state: &Arc<Mutex<ViewerState>>,
    region_selection: &RegionSelectionStore,
) {
    let mut state = state.lock().expect("viewer state lock");
    state.viewport_width_logical = f64::from(window.get_viewport_width());
    state.viewport_height_logical = f64::from(window.get_viewport_height());
    let Some(info) = state.info.clone() else {
        return;
    };
    match viewer::layout(
        &info,
        state.zoom,
        (state.pan_x_logical, state.pan_y_logical),
        (state.viewport_width_logical, state.viewport_height_logical),
        state.device_scale,
    ) {
        Ok(layout) => {
            window.set_image_x(layout.image_x_logical as f32);
            window.set_image_y(layout.image_y_logical as f32);
            refresh_region_overlay(window, &state, region_selection);
        }
        Err(error) => window.set_viewer_status(format!("Pan failed: {error}").into()),
    }
}

fn run_desktop_application() {
    let store = match dh_project::ProjectStore::from_os_app_data() {
        Ok(store) => store,
        Err(error) => {
            eprintln!("cannot initialize private application data: {error}");
            std::process::exit(1);
        }
    };

    let window = match dh_ui::new_main_window() {
        Ok(window) => window,
        Err(error) => {
            eprintln!("failed to create Document Harvester window: {error}");
            std::process::exit(1);
        }
    };

    let (renderer, pdfium_error): (Option<Arc<dyn PageRenderer>>, Option<String>) =
        match PdfiumRenderer::load_bundled() {
            Ok(pdfium) => (Some(Arc::new(pdfium)), None),
            Err(error) => {
                eprintln!("bundled PDFium renderer is unavailable: {error}");
                (None, Some(format!("PDFium library unavailable: {error}")))
            }
        };
    refresh_import_availability(&window, WorkspaceAccess::Writable, pdfium_error.as_deref());

    let viewer_state = Arc::new(Mutex::new(ViewerState::new()));
    let region_selection: RegionSelectionStore = Arc::new(Mutex::new(None));
    let region_drag: RegionDragStore = Arc::new(Mutex::new(None));
    let preview_workflow: PreviewWorkflowStore =
        Arc::new(Mutex::new(PreviewWorkflow::default()));
    let mut navigation = Navigation::new();
    match open_stored_project(&store) {
        Err(error) => {
            navigation.apply(NavigationEvent::ProjectNotOpened);
            window.set_status_message(
                format!("Private project state could not be read: {error}").into(),
            );
        }
        Ok(StoredProjectOpen::Opened(outcome)) => {
            present_open_project(
                &window,
                &mut navigation,
                &ViewerContext {
                    state: &viewer_state,
                    selection: &region_selection,
                },
                &outcome,
                renderer.as_ref(),
                pdfium_error.as_deref(),
                true,
                &store,
            );
        }
        Ok(StoredProjectOpen::WorkbookMissing) => {
            navigation.apply(NavigationEvent::ProjectNotOpened);
            window.set_status_message(
                "The last project's workbook is missing. Create a new project.".into(),
            );
        }
        Ok(StoredProjectOpen::NoProject) => {
            navigation.apply(NavigationEvent::NoProjectStored);
        }
    }
    apply_screen(&window, navigation);
    refresh_create_project(
        &window,
        window.get_project_name().as_str(),
        window.get_parent_directory().as_str(),
        false,
    );

    // Re-decide Create Project availability whenever the name changes.
    let weak_window = window.as_weak();
    window.on_project_form_changed(move |name, parent_directory| {
        if let Some(window) = weak_window.upgrade() {
            let busy = window.get_project_busy();
            refresh_create_project(&window, name.as_str(), parent_directory.as_str(), busy);
        }
    });

    let weak_window = window.as_weak();
    window.on_choose_parent_directory(move || {
        let weak_window = weak_window.clone();
        let _ = slint::spawn_local(async move {
            let selected = rfd::AsyncFileDialog::new()
                .set_title("Choose the parent folder for the new project")
                .pick_folder()
                .await;
            if let Some(folder) = selected {
                let parent = folder.path().to_string_lossy().to_string();
                let _ = weak_window.upgrade_in_event_loop(move |window| {
                    window.set_parent_directory(parent.into());
                    let busy = window.get_project_busy();
                    let name = window.get_project_name();
                    let parent_directory = window.get_parent_directory();
                    refresh_create_project(&window, name.as_str(), parent_directory.as_str(), busy);
                    window.set_status_message(
                        "Choose a project name, then create the project.".into(),
                    );
                });
            }
        });
    });

    let store_for_create = store.clone();
    let viewer_state_for_create = viewer_state.clone();
    let region_selection_for_create = region_selection.clone();
    let renderer_for_create = renderer.clone();
    let pdfium_error_for_create = pdfium_error.clone();
    let weak_window = window.as_weak();
    window.on_create_project(move |name, parent_directory| {
        let store = store_for_create.clone();
        let viewer_state = viewer_state_for_create.clone();
        let region_selection = region_selection_for_create.clone();
        let renderer = renderer_for_create.clone();
        let pdfium_error = pdfium_error_for_create.clone();
        let weak_window = weak_window.clone();
        if let Some(window) = weak_window.upgrade() {
            window.set_project_busy(true);
            refresh_create_project(&window, name.as_str(), parent_directory.as_str(), true);
            window.set_status_message("Creating project and blank workbook…".into());
        }
        let name = name.to_string();
        let parent_directory = PathBuf::from(parent_directory.as_str());

        std::thread::spawn(move || {
            let result = create_and_open_project(&store, &name, &parent_directory);
            match result {
                Ok(outcome) => {
                    let _ = weak_window.upgrade_in_event_loop(move |window| {
                        window.set_project_busy(false);
                        let mut navigation = navigation;
                        present_open_project(
                            &window,
                            &mut navigation,
                            &ViewerContext {
                                state: &viewer_state,
                                selection: &region_selection,
                            },
                            &outcome,
                            renderer.as_ref(),
                            pdfium_error.as_deref(),
                            false,
                            &store,
                        );
                        refresh_create_project(&window, "", "", false);
                    });
                }
                Err(error) => {
                    let message = error.to_string();
                    let mut navigation = navigation;
                    navigation.apply(NavigationEvent::ProjectNotOpened);
                    let _ = weak_window.upgrade_in_event_loop(move |window| {
                        window.set_project_busy(false);
                        apply_screen(&window, navigation);
                        let name = window.get_project_name();
                        let parent_directory = window.get_parent_directory();
                        refresh_create_project(
                            &window,
                            name.as_str(),
                            parent_directory.as_str(),
                            false,
                        );
                        window.set_status_message(message.into());
                    });
                }
            }
        });
    });

    let viewer_state_for_new = viewer_state.clone();
    let region_selection_for_new = region_selection.clone();
    let preview_workflow_for_new = preview_workflow.clone();
    let weak_window = window.as_weak();
    window.on_new_project(move || {
        let weak_window = weak_window.clone();
        let viewer_state = viewer_state_for_new.clone();
        let region_selection = region_selection_for_new.clone();
        let preview_workflow = preview_workflow_for_new.clone();
        let mut navigation = navigation;
        navigation.apply(NavigationEvent::NewProjectRequested);
        let _ = weak_window.upgrade_in_event_loop(move |window| {
            window.set_project_busy(false);
            window.set_project_name("".into());
            window.set_parent_directory("".into());
            window.set_workbook_path("".into());
            window.set_pdf_imported(false);
            window.set_workspace_writable(false);
            window.set_history_rows(Vec::<SharedString>::new().into());
            window.set_history_empty_message("No project is open.".into());
            window.set_history_busy(false);
            window.set_undo_enabled(false);
            window.set_undo_note("Open a project to view capture history.".into());
            preview_workflow.lock().expect("preview workflow lock").clear();
            refresh_preview_ui(&window, &preview_workflow);
            window.set_page_image(Image::default());
            *viewer_state.lock().expect("viewer state lock") = ViewerState::new();
            clear_region_selection(&window, &region_selection);
            apply_screen(&window, navigation);
            refresh_create_project(&window, "", "", false);
            window.set_status_message("Choose a parent folder and enter a project name.".into());
        });
    });

    // Import copies the chosen PDF into app data on a worker thread. After the
    // copy is recorded, only that private path is given to the renderer.
    let store_for_import = store.clone();
    let renderer_for_import = renderer.clone();
    let state_for_import = viewer_state.clone();
    let region_selection_for_import = region_selection.clone();
    let weak_window = window.as_weak();
    window.on_import_pdf(move || {
        let Some(renderer) = renderer_for_import.clone() else {
            return;
        };
        let store = store_for_import.clone();
        let state = state_for_import.clone();
        let region_selection = region_selection_for_import.clone();
        let weak_window = weak_window.clone();
        let _ = slint::spawn_local(async move {
            let selected = rfd::AsyncFileDialog::new()
                .set_title("Import a digital PDF")
                .add_filter("PDF", &["pdf"])
                .pick_file()
                .await;
            let Some(file) = selected else {
                return;
            };
            let source = file.path().to_path_buf();
            let project_id = state.lock().expect("viewer state lock").project_id.clone();
            let weak_worker = weak_window.clone();
            let state_worker = state.clone();
            let region_selection_worker = region_selection.clone();
            let store_worker = store.clone();
            std::thread::spawn(move || {
                let result = store_worker.import_pdf(&project_id, &source);
                let _ = weak_worker.upgrade_in_event_loop(move |window| match result {
                    Ok(copy) => {
                        {
                            let mut state = state_worker.lock().expect("viewer state lock");
                            state.pdf_path = Some(copy);
                            state.page_number = 1;
                            state.page_count = 0;
                            state.info = None;
                            state.zoom = 1.0;
                            state.pan_x_logical = 0.0;
                            state.pan_y_logical = 0.0;
                        }
                        clear_region_selection(&window, &region_selection_worker);
                        window.set_pdf_imported(true);
                        window
                            .set_status_message("PDF copied into private application data.".into());
                        start_render(
                            &window,
                            &state_worker,
                            &renderer,
                            &region_selection_worker,
                            true,
                        );
                    }
                    Err(error) => {
                        window.set_status_message(format!("PDF import failed: {error}").into())
                    }
                });
            });
        });
    });

    let renderer_for_pages = renderer.clone();
    let state_for_pages = viewer_state.clone();
    let region_selection_for_pages = region_selection.clone();
    let weak_window = window.as_weak();
    window.on_previous_page(move || {
        let Some(renderer) = renderer_for_pages.clone() else {
            return;
        };
        if let Some(window) = weak_window.upgrade() {
            let changed = state_for_pages
                .lock()
                .expect("viewer state lock")
                .move_page(-1);
            if changed {
                clear_region_selection(&window, &region_selection_for_pages);
                start_render(
                    &window,
                    &state_for_pages,
                    &renderer,
                    &region_selection_for_pages,
                    false,
                );
            }
        }
    });

    let renderer_for_pages = renderer.clone();
    let state_for_pages = viewer_state.clone();
    let region_selection_for_pages = region_selection.clone();
    let weak_window = window.as_weak();
    window.on_next_page(move || {
        let Some(renderer) = renderer_for_pages.clone() else {
            return;
        };
        if let Some(window) = weak_window.upgrade() {
            let changed = state_for_pages
                .lock()
                .expect("viewer state lock")
                .move_page(1);
            if changed {
                clear_region_selection(&window, &region_selection_for_pages);
                start_render(
                    &window,
                    &state_for_pages,
                    &renderer,
                    &region_selection_for_pages,
                    false,
                );
            }
        }
    });

    let renderer_for_zoom = renderer.clone();
    let state_for_zoom = viewer_state.clone();
    let region_selection_for_zoom = region_selection.clone();
    let weak_window = window.as_weak();
    window.on_zoom_in(move || {
        let Some(renderer) = renderer_for_zoom.clone() else {
            return;
        };
        if let Some(window) = weak_window.upgrade() {
            state_for_zoom
                .lock()
                .expect("viewer state lock")
                .change_zoom(1.25);
            start_render(
                &window,
                &state_for_zoom,
                &renderer,
                &region_selection_for_zoom,
                false,
            );
        }
    });

    let renderer_for_zoom = renderer.clone();
    let state_for_zoom = viewer_state.clone();
    let region_selection_for_zoom = region_selection.clone();
    let weak_window = window.as_weak();
    window.on_zoom_out(move || {
        let Some(renderer) = renderer_for_zoom.clone() else {
            return;
        };
        if let Some(window) = weak_window.upgrade() {
            state_for_zoom
                .lock()
                .expect("viewer state lock")
                .change_zoom(0.8);
            start_render(
                &window,
                &state_for_zoom,
                &renderer,
                &region_selection_for_zoom,
                false,
            );
        }
    });

    let renderer_for_fit = renderer.clone();
    let state_for_fit = viewer_state.clone();
    let region_selection_for_fit = region_selection.clone();
    let weak_window = window.as_weak();
    window.on_fit_page(move || {
        let Some(renderer) = renderer_for_fit.clone() else {
            return;
        };
        if let Some(window) = weak_window.upgrade() {
            state_for_fit
                .lock()
                .expect("viewer state lock")
                .pan_x_logical = 0.0;
            state_for_fit
                .lock()
                .expect("viewer state lock")
                .pan_y_logical = 0.0;
            start_render(
                &window,
                &state_for_fit,
                &renderer,
                &region_selection_for_fit,
                true,
            );
        }
    });

    let state_for_pan = viewer_state.clone();
    let region_selection_for_pan = region_selection.clone();
    let weak_window = window.as_weak();
    window.on_pan(move |dx, dy| {
        if let Some(window) = weak_window.upgrade() {
            state_for_pan
                .lock()
                .expect("viewer state lock")
                .pan(f64::from(dx), f64::from(dy));
            refresh_pan(&window, &state_for_pan, &region_selection_for_pan);
        }
    });

    let weak_window = window.as_weak();
    window.on_toggle_selection_mode(move || {
        let Some(window) = weak_window.upgrade() else {
            return;
        };
        let selecting = !window.get_selection_mode();
        window.set_selection_mode(selecting);
        window.set_viewer_status(
            if selecting {
                "Select region: drag to draw, drag inside to move, or drag a handle to resize. Out-of-page selections are rejected."
            } else {
                "Pan page: drag the page to move the view."
            }
            .into(),
        );
    });

    let state_for_region = viewer_state.clone();
    let selection_for_region = region_selection.clone();
    let drag_for_region = region_drag.clone();
    let weak_window = window.as_weak();
    window.on_region_pointer_down(move |kind, x, y| {
        let Some(window) = weak_window.upgrade() else {
            return;
        };
        let scale = state_for_region
            .lock()
            .expect("viewer state lock")
            .device_scale;
        let start_x_px = f64::from(x) * scale;
        let start_y_px = f64::from(y) * scale;
        let kind = kind.as_str().to_owned();
        let base_rect_px = if kind == "draw" {
            (start_x_px, start_y_px, start_x_px, start_y_px)
        } else {
            (
                f64::from(window.get_region_x()) * scale,
                f64::from(window.get_region_y()) * scale,
                f64::from(window.get_region_x() + window.get_region_width()) * scale,
                f64::from(window.get_region_y() + window.get_region_height()) * scale,
            )
        };
        *drag_for_region.lock().expect("region drag lock") = Some(RegionDrag {
            kind,
            start_x_px,
            start_y_px,
            base_rect_px,
        });
        if drag_for_region
            .lock()
            .expect("region drag lock")
            .as_ref()
            .is_some_and(|drag| drag.kind == "draw")
        {
            clear_region_selection(&window, &selection_for_region);
        }
    });

    let state_for_region = viewer_state.clone();
    let selection_for_region = region_selection.clone();
    let drag_for_region = region_drag.clone();
    let weak_window = window.as_weak();
    window.on_region_pointer_move(move |x, y| {
        let Some(drag) = drag_for_region.lock().expect("region drag lock").clone() else {
            return;
        };
        let Some(window) = weak_window.upgrade() else {
            return;
        };
        let scale = state_for_region
            .lock()
            .expect("viewer state lock")
            .device_scale;
        let rectangle_px =
            candidate_screen_rect_px(&drag, f64::from(x) * scale, f64::from(y) * scale);
        let state = state_for_region.lock().expect("viewer state lock");
        apply_region_candidate(&window, &state, &selection_for_region, rectangle_px);
    });

    let state_for_region = viewer_state.clone();
    let selection_for_region = region_selection.clone();
    let drag_for_region = region_drag.clone();
    let weak_window = window.as_weak();
    window.on_region_pointer_up(move |x, y| {
        let Some(drag) = drag_for_region.lock().expect("region drag lock").take() else {
            return;
        };
        let Some(window) = weak_window.upgrade() else {
            return;
        };
        let scale = state_for_region
            .lock()
            .expect("viewer state lock")
            .device_scale;
        let rectangle_px =
            candidate_screen_rect_px(&drag, f64::from(x) * scale, f64::from(y) * scale);
        let state = state_for_region.lock().expect("viewer state lock");
        apply_region_candidate(&window, &state, &selection_for_region, rectangle_px);
    });

    // Stage 4: create a review-only preview from the current accepted region.
    let store_for_preview = store.clone();
    let state_for_preview = viewer_state.clone();
    let selection_for_preview = region_selection.clone();
    let workflow_for_preview = preview_workflow.clone();
    let weak_window = window.as_weak();
    window.on_preview_region(move || {
        if let Some(window) = weak_window.upgrade() {
            start_preview_generation(
                &window,
                store_for_preview.clone(),
                state_for_preview.clone(),
                selection_for_preview.clone(),
                workflow_for_preview.clone(),
            );
        }
    });

    let workflow_for_table_selection = preview_workflow.clone();
    let weak_window = window.as_weak();
    window.on_table_selection_changed(move |table_id, checked| {
        if let Some(window) = weak_window.upgrade() {
            let changed = workflow_for_table_selection
                .lock()
                .expect("preview workflow lock")
                .set_table_checked(table_id.as_str(), checked);
            refresh_preview_ui(&window, &workflow_for_table_selection);
            if changed {
                window.set_status_message(
                    "Table selection updated. Partial-cell confirmation must be reviewed again."
                        .into(),
                );
            }
        }
    });

    let workflow_for_confirm = preview_workflow.clone();
    let weak_window = window.as_weak();
    window.on_confirm_partial_cells(move || {
        if let Some(window) = weak_window.upgrade() {
            let confirmed = workflow_for_confirm
                .lock()
                .expect("preview workflow lock")
                .confirm_partial_cells();
            refresh_preview_ui(&window, &workflow_for_confirm);
            if confirmed {
                window.set_status_message(
                    "Partial cells are confirmed as blank; their text will not be written."
                        .into(),
                );
            }
        }
    });

    let workflow_for_cancel = preview_workflow.clone();
    let weak_window = window.as_weak();
    window.on_cancel_preview(move || {
        if let Some(window) = weak_window.upgrade() {
            workflow_for_cancel
                .lock()
                .expect("preview workflow lock")
                .clear();
            refresh_preview_ui(&window, &workflow_for_cancel);
            window.set_status_message("Preview cancelled; workbook unchanged.".into());
        }
    });

    let store_for_approval = store.clone();
    let state_for_approval = viewer_state.clone();
    let selection_for_approval = region_selection.clone();
    let workflow_for_approval = preview_workflow.clone();
    let weak_window = window.as_weak();
    window.on_approve_preview(move || {
        let Some(window) = weak_window.upgrade() else {
            return;
        };
        if window.get_viewer_busy() {
            window.set_status_message(
                "Wait for the current page or preview operation to finish before approval."
                    .into(),
            );
            return;
        }
        let writable = window.get_workspace_writable();
        let pending = workflow_for_approval
            .lock()
            .expect("preview workflow lock")
            .pending
            .clone();
        let Some(pending) = pending else {
            window.set_status_message("There is no preview to approve.".into());
            refresh_preview_ui(&window, &workflow_for_approval);
            return;
        };

        let selected_region = *selection_for_approval
            .lock()
            .expect("region selection lock");
        let state_snapshot = state_for_approval
            .lock()
            .expect("viewer state lock")
            .clone();
        // Page navigation clears the visible region selection. Reapply the
        // approved preview's page-space rectangle to the current page so the
        // old preview is rejected and a fresh one can be generated. The user
        // still has to inspect and approve that replacement preview.
        let region = selected_region.unwrap_or_else(|| {
            let prior_region = pending.identity.region;
            *selection_for_approval
                .lock()
                .expect("region selection lock") = Some(prior_region);
            refresh_region_overlay(&window, &state_snapshot, &selection_for_approval);
            prior_region
        });
        let current_identity =
            current_preview_identity(&store_for_approval, &state_snapshot, region);
        match current_identity {
            Ok(current) if pending.identity.differs_from(&current) => {
                workflow_for_approval
                    .lock()
                    .expect("preview workflow lock")
                    .clear();
                refresh_preview_ui(&window, &workflow_for_approval);
                window.set_status_message(
                    "Stale preview rejected. A fresh preview is being generated; review and approve it again."
                        .into(),
                );
                start_preview_generation(
                    &window,
                    store_for_approval.clone(),
                    state_for_approval.clone(),
                    selection_for_approval.clone(),
                    workflow_for_approval.clone(),
                );
                return;
            }
            Ok(_) => {}
            Err(error) => {
                workflow_for_approval
                    .lock()
                    .expect("preview workflow lock")
                    .clear();
                refresh_preview_ui(&window, &workflow_for_approval);
                window.set_status_message(
                    format!("Preview freshness could not be verified; commit refused: {error}")
                        .into(),
                );
                return;
            }
        }

        let can_begin = workflow_for_approval
            .lock()
            .expect("preview workflow lock")
            .begin_commit(writable);
        if !can_begin {
            refresh_preview_ui(&window, &workflow_for_approval);
            window.set_status_message(
                if !writable {
                    "Commit is disabled because this project is read-only."
                } else if pending.partial_cell_count > 0 {
                    "Confirm that partial cells will remain blank before commit."
                } else {
                    "The preview is not ready or a commit attempt is already in progress."
                }
                .into(),
            );
            return;
        }
        let approved_preview = workflow_for_approval
            .lock()
            .expect("preview workflow lock")
            .pending
            .clone()
            .expect("approved preview remains pending");
        let approved_viewer_state = state_for_approval
            .lock()
            .expect("viewer state lock")
            .clone();
        refresh_preview_ui(&window, &workflow_for_approval);
        window.set_status_message("Explicit approval received; committing selected tables…".into());
        window.set_viewer_busy(true);

        let store = store_for_approval.clone();
        let history_store = store.clone();
        let project_id = approved_preview.identity.project_id.clone();
        let weak_window = window.as_weak();
        let workflow = workflow_for_approval.clone();
        std::thread::spawn(move || {
            let result = commit_capture_preview(&store, &approved_preview, &approved_viewer_state);
            let _ = weak_window.upgrade_in_event_loop(move |window| {
                window.set_viewer_busy(false);
                match result {
                    Ok(true) => {
                        workflow.lock().expect("preview workflow lock").clear();
                        refresh_preview_ui(&window, &workflow);
                        window.set_viewer_status(
                            "Capture committed through the verified workbook transaction.".into(),
                        );
                        window.set_status_message(
                            "Capture committed. The preview has been cleared.".into(),
                        );
                        refresh_history_async(&window, history_store.clone(), project_id.clone(),
                            window.get_workspace_writable());
                    }
                    Ok(false) => {
                        workflow.lock().expect("preview workflow lock").clear();
                        refresh_preview_ui(&window, &workflow);
                        window.set_viewer_status(
                            "The capture was already committed during journal recovery.".into(),
                        );
                        window.set_status_message(
                            "Recovered the existing capture; no duplicate was written.".into(),
                        );
                        refresh_history_async(&window, history_store.clone(), project_id.clone(),
                            window.get_workspace_writable());
                    }
                    Err(error) => {
                        // Keep the preview and approval state so a locked workbook
                        // can be retried without regenerating or duplicating it.
                        workflow
                            .lock()
                            .expect("preview workflow lock")
                            .finish_commit();
                        refresh_preview_ui(&window, &workflow);
                        let message = error.to_string();
                        if message.to_ascii_lowercase().contains("locked") {
                            window.set_viewer_status(
                                "Workbook is locked. Close Excel and use Retry commit; the approved preview is retained."
                                    .into(),
                            );
                            window.set_status_message(
                                "Commit deferred. Close Excel, then retry the pending preview."
                                    .into(),
                            );
                        } else {
                            window.set_viewer_status(
                                format!("Commit failed; approved preview retained: {message}")
                                    .into(),
                            );
                            window.set_status_message(
                                "The pending preview remains available for retry.".into(),
                            );
                        }
                    }
                }
            });
        });
    });


    // Latest-capture undo is an application intent; project and workbook work
    // stays on a worker thread and the UI receives only status/history data.
    let store_for_undo = store.clone();
    let state_for_undo = viewer_state.clone();
    let workflow_for_undo = preview_workflow.clone();
    let weak_window = window.as_weak();
    window.on_undo_last_capture(move || {
        let Some(window) = weak_window.upgrade() else { return; };
        if window.get_viewer_busy() || window.get_history_busy() {
            window.set_status_message("Wait for the current operation to finish before undo.".into());
            return;
        }
        if !window.get_workspace_writable() {
            window.set_status_message(navigation::READ_ONLY_NOTE.into());
            return;
        }
        if !window.get_undo_enabled() {
            window.set_status_message(window.get_undo_note());
            return;
        }
        let project_id = state_for_undo.lock().expect("viewer state lock").project_id.clone();
        if project_id.is_empty() {
            window.set_status_message("Undo is unavailable because no project is open.".into());
            return;
        }
        window.set_history_busy(true);
        window.set_undo_enabled(false);
        window.set_undo_note("Undoing the latest capture…".into());
        window.set_viewer_busy(true);
        window.set_status_message("Rebuilding the workbook without the latest capture…".into());
        let store = store_for_undo.clone();
        let history_store = store.clone();
        let weak_window = window.as_weak();
        let workflow = workflow_for_undo.clone();
        std::thread::spawn(move || {
            let result = store.undo_last_capture(&project_id, &dh_writer::XlsxWriter);
            let _ = weak_window.upgrade_in_event_loop(move |window| {
                window.set_viewer_busy(false);
                match result {
                    Ok(removed) => {
                        window.set_status_message(
                            format!("Undid capture {} from page {}.", removed.id, removed.page).into(),
                        );
                        window.set_viewer_status(
                            "Latest capture removed through the verified workbook transaction.".into(),
                        );
                    }
                    Err(error) => {
                        window.set_status_message(format!("Undo refused: {error}").into());
                    }
                }
                refresh_history_async(&window, history_store, project_id,
                    window.get_workspace_writable());
                // A changed workbook hash makes any pending preview stale under D16.
                refresh_preview_ui(&window, &workflow);
            });
        });
    });

    if let Err(error) = window.run() {
        eprintln!("Document Harvester desktop UI stopped: {error}");
        std::process::exit(1);
    }
}

// Recorded from `pdfinfo -box -f 1 -l 1` (independent oracle) on the owner
// fixtures. `pdftoppm -r 72` renders page 1 of each at 596 x 843 px.
const PDFIUM_FIXTURES: &[(&str, u32, f64, f64)] = &[
    ("01.pdf", 6, 595.5, 842.25),
    ("02.pdf", 7, 595.5, 842.25),
    ("03.pdf", 5, 595.5, 842.25),
];
const RENDER_WIDTH_PX: u32 = 596;
const RENDER_HEIGHT_PX: u32 = 843;

fn owner_pdf(name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join("..")
        .join("PDF_tests")
        .join(name)
}

/// Prove that the library co-located with the built executable loads and
/// renders page 1 of every owner PDF fixture at the independently recorded
/// size with non-blank pixels. This is a runtime packaging check, not owner
/// acceptance of extraction accuracy.
fn run_pdfium_self_test() -> i32 {
    let renderer = match PdfiumRenderer::load_bundled() {
        Ok(renderer) => renderer,
        Err(error) => {
            eprintln!("pdfium self-test failed: {error}");
            return 1;
        }
    };
    let library = dh_part1::renderer::bundled_library_path()
        .map(|path| path.to_string_lossy().to_string())
        .unwrap_or_else(|_| "<unknown>".to_owned());

    for (name, expected_pages, expected_width, expected_height) in PDFIUM_FIXTURES {
        let pdf_path = owner_pdf(name);
        if !pdf_path.is_file() {
            eprintln!(
                "pdfium self-test failed: owner fixture is missing: {}",
                pdf_path.display()
            );
            return 1;
        }
        let result = (|| -> dh_core::Result<(PageInfo, RenderedPage)> {
            let info = renderer.page_info(&pdf_path, 1)?;
            let page = renderer.render_page(&pdf_path, 1, RENDER_WIDTH_PX, RENDER_HEIGHT_PX)?;
            Ok((info, page))
        })();
        let (info, page) = match result {
            Ok(result) => result,
            Err(error) => {
                eprintln!("pdfium self-test failed for {name}: {error}");
                return 1;
            }
        };
        let non_white = page
            .pixels_rgba
            .as_chunks::<4>()
            .0
            .iter()
            .filter(|rgba| rgba.iter().any(|channel| *channel < 250))
            .count();
        if info.page_count != *expected_pages
            || info.width_points != *expected_width
            || info.height_points != *expected_height
            || page.width_px != RENDER_WIDTH_PX
            || page.height_px != RENDER_HEIGHT_PX
            || non_white <= 1000
        {
            eprintln!(
                "pdfium self-test failed for {name}: pages={} size={}x{} render={}x{} non_white={non_white}",
                info.page_count,
                info.width_points,
                info.height_points,
                page.width_px,
                page.height_px
            );
            return 1;
        }
        println!(
            "pdfium self-test: file={name} library='{library}' pages={} page_size={}x{} render={}x{} non_white={non_white}",
            info.page_count,
            info.width_points,
            info.height_points,
            page.width_px,
            page.height_px
        );
    }
    0
}

fn main() {
    let args: Vec<String> = env::args().skip(1).collect();

    if args.is_empty() {
        run_desktop_application();
        return;
    }
    if args.len() == 1 && args[0] == "--pdfium-self-test" {
        std::process::exit(run_pdfium_self_test());
    }

    let options = match parse(&args) {
        Ok(options) => options,
        Err(message) => {
            eprintln!("{message}\n{}", usage());
            std::process::exit(2);
        }
    };

    println!("Document Harvester 0.1.0-scaffold — stub entry point");
    println!("part: {:?}", options.part);
    println!("input: {:?}", options.input);
    println!("output: {:?}", options.output);
    println!("triage only: {}", options.triage_only);
    println!("force: {}", options.force);

    match options.column_order.as_deref() {
        Some(selection) => match dh_core::config::ColumnOrder::from_selection(selection) {
            Ok(order) => println!("column order: {}", order.as_str()),
            Err(error) => {
                eprintln!("{error}");
                std::process::exit(2);
            }
        },
        None => println!("column order: not selected (required before each job)"),
    }

    println!(
        "available digital lanes: {}",
        dh_part1::batch::available_lane_count()
    );
    println!(
        "ui projection rows: {}",
        dh_ui::UiBridge::new()
            .shared_state()
            .snapshot()
            .progress
            .len()
    );
    println!(
        "region export gate: {:?}",
        dh_part3::alerts::ExportGate::Allowed
    );
    println!("status: no extraction performed — Phase 0 evidence gates are open");
}

#[cfg(test)]
mod tests {
    use super::*;
    use dh_ir::BBox;
    use dh_part1::renderer::PageInfo;

    fn args(values: &[&str]) -> Vec<String> {
        values.iter().map(|value| (*value).to_owned()).collect()
    }

    #[test]
    fn parses_full_job() {
        let options = parse(&args(&[
            "--part",
            "1",
            "--input",
            "in.pdf",
            "--output",
            "out.xlsx",
            "--column-order",
            "left_to_right",
            "--triage-only",
        ]))
        .unwrap();
        assert_eq!(options.part, Some(1));
        assert!(options.triage_only);
        assert_eq!(options.column_order.as_deref(), Some("left_to_right"));
    }

    #[test]
    fn rejects_unknown_part_and_missing_value() {
        assert!(parse(&args(&["--part", "9"])).is_err());
        assert!(parse(&args(&["--input"])).is_err());
    }

    #[test]
    fn self_test_flag_is_not_a_regular_cli_job() {
        assert!(parse(&args(&["--pdfium-self-test"])).is_err());
    }

    #[test]
    fn create_and_launch_restore_both_use_the_recovery_aware_open_entry_point() {
        let unique = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|duration| duration.as_nanos())
            .unwrap_or_default();
        let root = std::env::temp_dir().join(format!(
            "dh-app-open-routes-{}-{unique}",
            std::process::id()
        ));
        let parent = root.join("projects");
        std::fs::create_dir_all(&parent).expect("create project parent");
        let store = dh_project::ProjectStore::new(root.join("private-app-data"));

        let created =
            create_and_open_project(&store, "Routes", &parent).expect("create and open project");
        assert!(
            matches!(created, dh_project::ProjectOpenOutcome::Ready(_)),
            "a new project opens writable through the recovery entry point"
        );
        assert_eq!(
            created.project().name,
            "Routes",
            "the open outcome carries the created project"
        );

        let restored = open_stored_project(&store).expect("launch restore");
        match restored {
            StoredProjectOpen::Opened(outcome) => {
                assert_eq!(outcome.project().id, created.project().id)
            }
            _ => panic!("launch restore must route the stored project through open_project"),
        }

        std::fs::remove_dir_all(root).expect("remove test directory");
    }

    fn close(actual: f64, expected: f64, label: &str) {
        assert!(
            (actual - expected).abs() <= 1e-6,
            "{label}: actual={actual}, expected={expected}"
        );
    }

    fn assert_rect(actual: BBox, expected: BBox, label: &str) {
        close(actual.x0, expected.x0, &format!("{label}.x0"));
        close(actual.y0, expected.y0, &format!("{label}.y0"));
        close(actual.x1, expected.x1, &format!("{label}.x1"));
        close(actual.y1, expected.y1, &format!("{label}.y1"));
    }

    fn region_page_box() -> BBox {
        BBox {
            x0: 10.0,
            y0: 20.0,
            x1: 210.0,
            y1: 120.0,
        }
    }

    fn region_page_info(rotation_degrees: u16) -> PageInfo {
        let page_box = region_page_box();
        PageInfo {
            page_count: 2,
            page_number: 1,
            width_points: page_box.x1 - page_box.x0,
            height_points: page_box.y1 - page_box.y0,
            rotation_degrees,
            page_box,
        }
    }

    fn region_viewer_state(
        zoom: f64,
        rotation_degrees: u16,
        pan: (f64, f64),
        viewport: (f64, f64),
        device_scale: f64,
    ) -> ViewerState {
        let mut state = ViewerState::new();
        state.page_number = 1;
        state.info = Some(region_page_info(rotation_degrees));
        state.zoom = zoom;
        state.pan_x_logical = pan.0;
        state.pan_y_logical = pan.1;
        state.viewport_width_logical = viewport.0;
        state.viewport_height_logical = viewport.1;
        state.device_scale = device_scale;
        state
    }

    /// Independent placement oracle: the centred plus panned page origin in
    /// logical viewport pixels. This mirrors the documented viewer placement,
    /// but does not call `viewer::layout` or `PageTransform`.
    fn independent_origin_logical(
        rotation_degrees: u16,
        zoom: f64,
        pan: (f64, f64),
        viewport: (f64, f64),
    ) -> (f64, f64) {
        let page = region_page_box();
        let page_width = page.x1 - page.x0;
        let page_height = page.y1 - page.y0;
        let (image_width, image_height) = if rotation_degrees == 90 || rotation_degrees == 270 {
            (page_height * zoom, page_width * zoom)
        } else {
            (page_width * zoom, page_height * zoom)
        };
        (
            (viewport.0 - image_width) / 2.0 + pan.0,
            (viewport.1 - image_height) / 2.0 + pan.1,
        )
    }

    /// Independent forward oracle from PDF user space to overlay-local logical
    /// pixels. Quarter-turn coordinates are written out by hand so the test
    /// never uses the transform it is checking.
    fn independent_logical_screen_rect(
        rotation_degrees: u16,
        zoom: f64,
        pan: (f64, f64),
        viewport: (f64, f64),
        page_rect: BBox,
    ) -> (f64, f64, f64, f64) {
        let page = region_page_box();
        let width = page.x1 - page.x0;
        let height = page.y1 - page.y0;
        let (origin_x, origin_y) =
            independent_origin_logical(rotation_degrees, zoom, pan, viewport);
        let corners = [
            (page_rect.x0, page_rect.y0),
            (page_rect.x0, page_rect.y1),
            (page_rect.x1, page_rect.y0),
            (page_rect.x1, page_rect.y1),
        ];
        let mut xs = Vec::new();
        let mut ys = Vec::new();
        for (x, y) in corners {
            let u = x - page.x0;
            let v = page.y1 - y;
            let (rotated_x, rotated_y) = match rotation_degrees {
                0 => (u, v),
                90 => (height - v, u),
                180 => (width - u, height - v),
                270 => (v, width - u),
                other => panic!("unexpected rotation: {other}"),
            };
            xs.push(origin_x + rotated_x * zoom);
            ys.push(origin_y + rotated_y * zoom);
        }
        (
            xs.iter().copied().fold(f64::INFINITY, f64::min),
            ys.iter().copied().fold(f64::INFINITY, f64::min),
            xs.iter().copied().fold(f64::NEG_INFINITY, f64::max),
            ys.iter().copied().fold(f64::NEG_INFINITY, f64::max),
        )
    }

    #[test]
    fn region_draw_intent_matches_an_independent_oracle_at_all_zooms_rotations_and_pans() {
        let expected_page = BBox {
            x0: 30.0,
            y0: 40.0,
            x1: 90.0,
            y1: 80.0,
        };
        let viewport = (900.0, 600.0);
        let device_scale = 2.0;

        for zoom in [1.0, 2.5, 4.0] {
            for rotation in [0, 90, 180, 270] {
                for pan in [(0.0, 0.0), (25.0, -15.0), (-40.0, 30.0)] {
                    let state = region_viewer_state(zoom, rotation, pan, viewport, device_scale);
                    let logical = independent_logical_screen_rect(
                        rotation,
                        zoom,
                        pan,
                        viewport,
                        expected_page,
                    );
                    let physical = (
                        logical.0 * device_scale,
                        logical.1 * device_scale,
                        logical.2 * device_scale,
                        logical.3 * device_scale,
                    );
                    let drag = RegionDrag {
                        kind: "draw".to_owned(),
                        start_x_px: physical.0,
                        start_y_px: physical.1,
                        base_rect_px: (physical.0, physical.1, physical.0, physical.1),
                    };
                    let candidate = candidate_screen_rect_px(&drag, physical.2, physical.3);
                    let actual = page_rect_from_screen_px(&state, candidate)
                        .unwrap_or_else(|error| panic!("{zoom} {rotation} {pan:?}: {error}"));
                    assert_rect(
                        actual,
                        expected_page,
                        &format!("zoom={zoom} rotation={rotation} pan={pan:?}"),
                    );
                }
            }
        }
    }

    #[test]
    fn region_drag_intents_move_and_resize_every_handle() {
        let base = RegionDrag {
            kind: "move".to_owned(),
            start_x_px: 120.0,
            start_y_px: 120.0,
            base_rect_px: (100.0, 100.0, 200.0, 200.0),
        };
        let expectations = [
            ("move", (120.0, 110.0, 220.0, 210.0)),
            ("left", (120.0, 100.0, 200.0, 200.0)),
            ("top", (100.0, 110.0, 200.0, 200.0)),
            ("top-left", (120.0, 110.0, 200.0, 200.0)),
            ("right", (100.0, 100.0, 220.0, 200.0)),
            ("top-right", (100.0, 110.0, 220.0, 200.0)),
            ("bottom", (100.0, 100.0, 200.0, 210.0)),
            ("bottom-right", (100.0, 100.0, 220.0, 210.0)),
            ("bottom-left", (120.0, 100.0, 200.0, 210.0)),
        ];
        for (kind, expected) in expectations {
            let drag = RegionDrag {
                kind: kind.to_owned(),
                ..base.clone()
            };
            let actual = candidate_screen_rect_px(&drag, 140.0, 130.0);
            close(actual.0, expected.0, &format!("{kind}.x0"));
            close(actual.1, expected.1, &format!("{kind}.y0"));
            close(actual.2, expected.2, &format!("{kind}.x1"));
            close(actual.3, expected.3, &format!("{kind}.y1"));
        }

        let draw = RegionDrag {
            kind: "draw".to_owned(),
            start_x_px: 100.0,
            start_y_px: 100.0,
            base_rect_px: (100.0, 100.0, 100.0, 100.0),
        };
        let actual = candidate_screen_rect_px(&draw, 140.0, 130.0);
        assert_eq!(actual, (100.0, 100.0, 140.0, 130.0));
    }

    #[test]
    fn selection_drawn_at_zoom_2_5_reads_back_at_zoom_1_in_page_coordinates() {
        let expected_page = BBox {
            x0: 30.0,
            y0: 40.0,
            x1: 90.0,
            y1: 80.0,
        };
        let viewport = (900.0, 600.0);
        let device_scale = 2.0;
        let rotation = 90;
        let pan = (25.0, -15.0);

        let mut state = region_viewer_state(2.5, rotation, pan, viewport, device_scale);
        let logical_at_2_5 =
            independent_logical_screen_rect(rotation, 2.5, pan, viewport, expected_page);
        let physical_at_2_5 = (
            logical_at_2_5.0 * device_scale,
            logical_at_2_5.1 * device_scale,
            logical_at_2_5.2 * device_scale,
            logical_at_2_5.3 * device_scale,
        );
        let drawn_page = page_rect_from_screen_px(&state, physical_at_2_5).expect("draw at 2.5x");
        assert_rect(drawn_page, expected_page, "drawn page rectangle");

        state.zoom = 1.0;
        let (transform, _) = current_page_transform(&state).expect("transform at 1x");
        let logical_at_1 = logical_rect_from_page_rect(&transform, drawn_page)
            .expect("project stored selection at 1x");
        let independent_at_1 =
            independent_logical_screen_rect(rotation, 1.0, pan, viewport, expected_page);
        close(logical_at_1.0, independent_at_1.0, "1x projection.x0");
        close(logical_at_1.1, independent_at_1.1, "1x projection.y0");
        close(logical_at_1.2, independent_at_1.2, "1x projection.x1");
        close(logical_at_1.3, independent_at_1.3, "1x projection.y1");
        assert!(
            (logical_at_2_5.2 - logical_at_2_5.0 - (logical_at_1.2 - logical_at_1.0)).abs() > 1.0,
            "re-projection must rescale the overlay rectangle"
        );

        let physical_at_1 = (
            logical_at_1.0 * device_scale,
            logical_at_1.1 * device_scale,
            logical_at_1.2 * device_scale,
            logical_at_1.3 * device_scale,
        );
        let reread_page = page_rect_from_screen_px(&state, physical_at_1).expect("read at 1x");
        assert_rect(reread_page, drawn_page, "page rectangle after zoom change");
    }

    #[test]
    fn invalid_region_intents_are_rejected_with_an_explicit_message() {
        let viewport = (900.0, 600.0);
        let state = region_viewer_state(1.0, 0, (0.0, 0.0), viewport, 1.0);
        let (origin_x, origin_y) = independent_origin_logical(0, 1.0, (0.0, 0.0), viewport);

        let zero_area = (
            origin_x + 20.0,
            origin_y + 10.0,
            origin_x + 20.0,
            origin_y + 50.0,
        );
        let error = page_rect_from_screen_px(&state, zero_area).unwrap_err();
        assert!(format!("{error:?}").contains("positive width and height"));

        let non_finite = (origin_x + 10.0, f64::NAN, origin_x + 50.0, origin_y + 40.0);
        let error = page_rect_from_screen_px(&state, non_finite).unwrap_err();
        assert!(format!("{error:?}").contains("finite coordinates"));

        let outside_page = (
            origin_x + 201.0,
            origin_y + 10.0,
            origin_x + 240.0,
            origin_y + 40.0,
        );
        let error = page_rect_from_screen_px(&state, outside_page).unwrap_err();
        assert!(format!("{error:?}").contains("outside the effective PDF page box"));
    }
    #[test]
    fn current_preview_identity_binds_page_region_and_workbook_hash() {
        let unique = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|duration| duration.as_nanos())
            .unwrap_or_default();
        let root = std::env::temp_dir().join(format!(
            "dh-app-preview-identity-{}-{unique}",
            std::process::id()
        ));
        let parent = root.join("projects");
        std::fs::create_dir_all(&parent).expect("create project parent");
        let store = dh_project::ProjectStore::new(root.join("private-app-data"));
        let project = store
            .create_project("Preview Identity", &parent, &dh_writer::XlsxWriter)
            .expect("create project");
        let source = root.join("source.pdf");
        std::fs::write(&source, b"%PDF-1.4\n% synthetic identity fixture")
            .expect("write source file");
        let imported = store
            .import_pdf(&project.id, &source)
            .expect("import source PDF");
        let mut state = ViewerState::new();
        state.project_id = project.id.clone();
        state.pdf_path = Some(imported);
        state.page_number = 2;
        let region = BBox {
            x0: 10.0,
            y0: 20.0,
            x1: 80.0,
            y1: 60.0,
        };
        let original = current_preview_identity(&store, &state, region)
            .expect("build preview identity");

        state.page_number = 3;
        assert!(
            original.differs_from(
                &current_preview_identity(&store, &state, region).expect("page identity")
            ),
            "page changes invalidate the preview"
        );
        state.page_number = 2;
        let changed_region = BBox {
            x0: 11.0,
            ..region
        };
        assert!(
            original.differs_from(
                &current_preview_identity(&store, &state, changed_region)
                    .expect("region identity")
            ),
            "region changes invalidate the preview"
        );

        std::fs::write(&project.workbook_path, b"externally changed workbook")
            .expect("mutate workbook bytes");
        assert!(
            original.differs_from(
                &current_preview_identity(&store, &state, region)
                    .expect("changed workbook identity")
            ),
            "workbook hash changes invalidate the preview"
        );
        std::fs::remove_dir_all(root).expect("remove identity test directory");
    }

    #[test]
    fn direct_commit_with_zero_checked_tables_is_refused() {
        let unique = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|duration| duration.as_nanos())
            .unwrap_or_default();
        let root = std::env::temp_dir().join(format!(
            "dh-app-zero-table-commit-{}-{unique}",
            std::process::id()
        ));
        let parent = root.join("projects");
        std::fs::create_dir_all(&parent).expect("create project parent");
        let store = dh_project::ProjectStore::new(root.join("private-app-data"));
        let project = store
            .create_project("Zero Table Commit", &parent, &dh_writer::XlsxWriter)
            .expect("create project");
        let region = BBox { x0: 0.0, y0: 0.0, x1: 100.0, y1: 50.0 };
        let preview = CapturePreview::from_region_result(
            PreviewIdentity {
                project_id: project.id.clone(),
                source_pdf_path: String::new(),
                source_pdf_sha256: String::new(),
                workbook_path: project.workbook_path.to_string_lossy().to_string(),
                workbook_sha256: dh_writer::file_sha256(&project.workbook_path)
                    .expect("hash initial workbook"),
                page: 1,
                region,
                policy_version: preview::PREVIEW_POLICY_VERSION.to_owned(),
            },
            dh_ir::RegionResult {
                page: 1,
                region,
                tables: Vec::new(),
                unassigned_text: Vec::new(),
                warnings: Vec::new(),
            },
            &[],
        );

        let error = commit_capture_preview(&store, &preview)
            .expect_err("a direct backend request with no checked tables must be refused");
        assert!(error.to_string().contains("select at least one reconstructed table"));
        assert!(store.capture_list(&project.id).expect("read captures").is_empty());
        std::fs::remove_dir_all(root).expect("remove zero-table test directory");
    }


}
