mod navigation;
mod viewer;

use std::env;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};

use dh_part1::renderer::{PageInfo, PageRenderer, PdfiumRenderer, RenderedPage};
use slint::{ComponentHandle, Image, Rgba8Pixel, SharedPixelBuffer};

use navigation::{
    availability, empty_state_message, Action, Availability, Navigation, NavigationEvent,
    ProjectForm, Screen,
};
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
fn refresh_pan(window: &dh_ui::MainWindow, state: &Arc<Mutex<ViewerState>>) {
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

    let renderer: Option<Arc<dyn PageRenderer>> = match PdfiumRenderer::load_bundled() {
        Ok(pdfium) => Some(Arc::new(pdfium)),
        Err(error) => {
            eprintln!("bundled PDFium renderer is unavailable: {error}");
            window.set_import_pdf_enabled(false);
            window.set_import_pdf_note(format!("PDFium library unavailable: {error}").into());
            None
        }
    };
    if renderer.is_some() {
        match availability(
            Screen::ProjectWorkspace,
            Action::ImportPdf,
            ProjectForm::default(),
        ) {
            Availability::Enabled => {
                window.set_import_pdf_enabled(true);
                window.set_import_pdf_note(navigation::IMPORT_PDF_NOTE.into());
            }
            Availability::Disabled(reason) => {
                window.set_import_pdf_enabled(false);
                window.set_import_pdf_note(reason.into());
            }
            Availability::Hidden => window.set_import_pdf_enabled(false),
        }
    }

    let viewer_state = Arc::new(Mutex::new(ViewerState::new()));
    let mut navigation = Navigation::new();
    match store.current_project() {
        Err(error) => {
            navigation.apply(NavigationEvent::ProjectNotOpened);
            window.set_status_message(
                format!("Private project state could not be read: {error}").into(),
            );
        }
        Ok(Some(project)) if project.workbook_path.is_file() => {
            navigation.apply(NavigationEvent::ProjectOpened);
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
            window
                .set_status_message("Last project restored from private application data.".into());
            {
                let mut state = viewer_state.lock().expect("viewer state lock");
                state.project_id = project.id.clone();
                if let Some(copy) = project.imported_pdf_path {
                    if copy.is_file() {
                        state.pdf_path = Some(copy);
                        window.set_pdf_imported(true);
                    } else {
                        window.set_viewer_status(
                            "The imported PDF copy is missing; import the PDF again.".into(),
                        );
                    }
                }
            }
            if window.get_pdf_imported() {
                if let Some(renderer) = &renderer {
                    start_render(&window, &viewer_state, renderer, true);
                }
            }
        }
        Ok(Some(_)) => {
            navigation.apply(NavigationEvent::ProjectNotOpened);
            window.set_status_message(
                "The last project's workbook is missing. Create a new project.".into(),
            );
        }
        Ok(None) => {
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
    let weak_window = window.as_weak();
    window.on_create_project(move |name, parent_directory| {
        let store = store_for_create.clone();
        let viewer_state = viewer_state_for_create.clone();
        let weak_window = weak_window.clone();
        if let Some(window) = weak_window.upgrade() {
            window.set_project_busy(true);
            refresh_create_project(&window, name.as_str(), parent_directory.as_str(), true);
            window.set_status_message("Creating project and blank workbook…".into());
        }
        let name = name.to_string();
        let parent_directory = PathBuf::from(parent_directory.as_str());

        std::thread::spawn(move || {
            let result = store.create_project(&name, &parent_directory, &dh_writer::XlsxWriter);
            match result {
                Ok(project) => {
                    let project_name = project.name;
                    let project_id = project.id;
                    let workbook_path = project.workbook_path.to_string_lossy().to_string();
                    let mut navigation = navigation;
                    navigation.apply(NavigationEvent::ProjectOpened);
                    let _ = weak_window.upgrade_in_event_loop(move |window| {
                        window.set_project_name(project_name.into());
                        window.set_workbook_path(workbook_path.into());
                        window.set_project_busy(false);
                        window.set_pdf_imported(false);
                        viewer_state.lock().expect("viewer state lock").project_id = project_id;
                        apply_screen(&window, navigation);
                        refresh_create_project(&window, "", "", false);
                        window.set_status_message("Project and blank workbook created.".into());
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
    let weak_window = window.as_weak();
    window.on_new_project(move || {
        let weak_window = weak_window.clone();
        let viewer_state = viewer_state_for_new.clone();
        let mut navigation = navigation;
        navigation.apply(NavigationEvent::NewProjectRequested);
        let _ = weak_window.upgrade_in_event_loop(move |window| {
            window.set_project_busy(false);
            window.set_project_name("".into());
            window.set_parent_directory("".into());
            window.set_workbook_path("".into());
            window.set_pdf_imported(false);
            window.set_page_image(Image::default());
            *viewer_state.lock().expect("viewer state lock") = ViewerState::new();
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
    let weak_window = window.as_weak();
    window.on_import_pdf(move || {
        let Some(renderer) = renderer_for_import.clone() else {
            return;
        };
        let store = store_for_import.clone();
        let state = state_for_import.clone();
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
                        window.set_pdf_imported(true);
                        window
                            .set_status_message("PDF copied into private application data.".into());
                        start_render(&window, &state_worker, &renderer, true);
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
                start_render(&window, &state_for_pages, &renderer, false);
            }
        }
    });

    let renderer_for_pages = renderer.clone();
    let state_for_pages = viewer_state.clone();
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
                start_render(&window, &state_for_pages, &renderer, false);
            }
        }
    });

    let renderer_for_zoom = renderer.clone();
    let state_for_zoom = viewer_state.clone();
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
            start_render(&window, &state_for_zoom, &renderer, false);
        }
    });

    let renderer_for_zoom = renderer.clone();
    let state_for_zoom = viewer_state.clone();
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
            start_render(&window, &state_for_zoom, &renderer, false);
        }
    });

    let renderer_for_fit = renderer.clone();
    let state_for_fit = viewer_state.clone();
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
            start_render(&window, &state_for_fit, &renderer, true);
        }
    });

    let state_for_pan = viewer_state.clone();
    let weak_window = window.as_weak();
    window.on_pan(move |dx, dy| {
        if let Some(window) = weak_window.upgrade() {
            state_for_pan
                .lock()
                .expect("viewer state lock")
                .pan(f64::from(dx), f64::from(dy));
            refresh_pan(&window, &state_for_pan);
        }
    });

    if let Err(error) = window.run() {
        eprintln!("Document Harvester desktop UI stopped: {error}");
        std::process::exit(1);
    }
}

/// Prove that the library co-located with the built executable loads, opens a
/// generated PDF and returns non-blank pixels. This is a smoke test, not
/// ground truth for any real PDF.
fn run_pdfium_self_test() -> i32 {
    let renderer = match PdfiumRenderer::load_bundled() {
        Ok(renderer) => renderer,
        Err(error) => {
            eprintln!("pdfium self-test failed: {error}");
            return 1;
        }
    };
    let directory = env::temp_dir().join(format!("dh-pdfium-self-test-{}", std::process::id()));
    if let Err(error) = std::fs::create_dir_all(&directory) {
        eprintln!("pdfium self-test failed: {error}");
        return 1;
    }
    let pdf_path = directory.join("generated.pdf");
    if let Err(error) = std::fs::write(&pdf_path, dh_part1::renderer::generated_smoke_pdf()) {
        eprintln!("pdfium self-test failed: {error}");
        return 1;
    }
    let result = (|| -> dh_core::Result<(PageInfo, RenderedPage)> {
        let info = renderer.page_info(&pdf_path, 1)?;
        let page = renderer.render_page(&pdf_path, 1, 200, 100)?;
        Ok((info, page))
    })();
    let _ = std::fs::remove_dir_all(&directory);
    match result {
        Ok((info, page)) if page.pixels_rgba.iter().any(|value| *value > 128) => {
            println!(
                "pdfium self-test: library='{}' pages={} rendered={}x{} bytes={}",
                dh_part1::renderer::bundled_library_path()
                    .map(|path| path.to_string_lossy().to_string())
                    .unwrap_or_else(|_| "<unknown>".to_owned()),
                info.page_count,
                page.width_px,
                page.height_px,
                page.pixels_rgba.len()
            );
            0
        }
        Ok(_) => {
            eprintln!("pdfium self-test failed: the generated page rendered blank");
            1
        }
        Err(error) => {
            eprintln!("pdfium self-test failed: {error}");
            1
        }
    }
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
}
