use std::env;
use std::path::PathBuf;

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

    match store.current_project() {
        Err(error) => {
            window.set_status_message(
                format!("Private project state could not be read: {error}").into(),
            );
        }
        Ok(Some(project)) if project.workbook_path.is_file() => {
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
            window.set_project_created(true);
            window
                .set_status_message("Last project restored from private application data.".into());
        }
        Ok(Some(_)) => {
            window.set_status_message(
                "The last project's workbook is missing. Create a new project or locate the original workbook.".into(),
            );
        }
        Ok(None) => {}
    }

    let weak_window = window.as_weak();
    window.on_choose_parent_directory(move || {
        let weak_window = weak_window.clone();
        slint::spawn_local(async move {
            let selected = rfd::AsyncFileDialog::new()
                .set_title("Choose the parent folder for the new project")
                .pick_folder()
                .await;
            if let Some(folder) = selected {
                let parent = folder.path().to_string_lossy().to_string();
                let _ = weak_window.upgrade_in_event_loop(move |window| {
                    window.set_parent_directory(parent.into());
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
                    let _ = weak_window.upgrade_in_event_loop(move |window| {
                        window.set_project_name(project_name.into());
                        window.set_workbook_path(workbook_path.into());
                        window.set_project_created(true);
                        window.set_project_busy(false);
                        window.set_status_message("Project and blank workbook created.".into());
                    });
                }
                Err(error) => {
                    let message = error.to_string();
                    let _ = weak_window.upgrade_in_event_loop(move |window| {
                        window.set_project_busy(false);
                        window.set_status_message(message.into());
                    });
                }
            }
        });
    });

    let weak_window = window.as_weak();
    window.on_create_another_project(move || {
        let weak_window = weak_window.clone();
        let _ = weak_window.upgrade_in_event_loop(|window| {
            window.set_project_created(false);
            window.set_project_busy(false);
            window.set_project_name("".into());
            window.set_parent_directory("".into());
            window.set_workbook_path("".into());
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
