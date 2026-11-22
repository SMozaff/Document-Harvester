mod navigation;

use std::env;
use std::path::PathBuf;

use slint::ComponentHandle;

use navigation::{
    availability, empty_state_message, Action, Availability, Navigation, NavigationEvent,
    ProjectForm, Screen,
};

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
    "usage: document-harvester [--part 1|2|3] [--input PATH] [--output PATH] \\\n       --column-order right_to_left|left_to_right [--triage-only] [--force]"
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

    // PDF import belongs to Stage 2 and does not exist yet, so the control is
    // reported as disabled with an explanation instead of being a dead button.
    match availability(
        Screen::ProjectWorkspace,
        Action::ImportPdf,
        ProjectForm::default(),
    ) {
        Availability::Enabled => window.set_import_pdf_enabled(true),
        Availability::Disabled(reason) => {
            window.set_import_pdf_enabled(false);
            window.set_import_pdf_note(reason.into());
        }
        Availability::Hidden => window.set_import_pdf_enabled(false),
    }

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
    let weak_window = window.as_weak();
    window.on_create_project(move |name, parent_directory| {
        let store = store_for_create.clone();
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
                    let workbook_path = project.workbook_path.to_string_lossy().to_string();
                    let mut navigation = navigation;
                    navigation.apply(NavigationEvent::ProjectOpened);
                    let _ = weak_window.upgrade_in_event_loop(move |window| {
                        window.set_project_name(project_name.into());
                        window.set_workbook_path(workbook_path.into());
                        window.set_project_busy(false);
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

    let weak_window = window.as_weak();
    window.on_new_project(move || {
        let weak_window = weak_window.clone();
        let mut navigation = navigation;
        navigation.apply(NavigationEvent::NewProjectRequested);
        let _ = weak_window.upgrade_in_event_loop(move |window| {
            window.set_project_busy(false);
            window.set_project_name("".into());
            window.set_parent_directory("".into());
            window.set_workbook_path("".into());
            apply_screen(&window, navigation);
            refresh_create_project(&window, "", "", false);
            window.set_status_message("Choose a parent folder and enter a project name.".into());
        });
    });

    if let Err(error) = window.run() {
        eprintln!("Document Harvester desktop UI stopped: {error}");
        std::process::exit(1);
    }
}

fn main() {
    let args: Vec<String> = env::args().skip(1).collect();

    if args.is_empty() {
        run_desktop_application();
        return;
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
}
