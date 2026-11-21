use std::{
    fs::{self, OpenOptions},
    io::Write,
    path::{Path, PathBuf},
    time::{SystemTime, UNIX_EPOCH},
};

use dh_core::{DhError, Result};
use dh_writer::XlsxWriter;
use serde::{Deserialize, Serialize};

const STATE_FILE_NAME: &str = "projects.json";
const STATE_VERSION: u32 = 1;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProjectRecord {
    pub id: String,
    pub name: String,
    pub project_directory: PathBuf,
    pub workbook_path: PathBuf,
    pub created_at_unix_ms: u128,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct ProjectIndex {
    version: u32,
    projects: Vec<ProjectRecord>,
    current_project_id: Option<String>,
}

impl Default for ProjectIndex {
    fn default() -> Self {
        Self {
            version: STATE_VERSION,
            projects: Vec::new(),
            current_project_id: None,
        }
    }
}

/// Stores project metadata in the OS application-data area.
///
/// The workbook and its project directory are kept at the user-selected
/// location. No project metadata or sidecar file is written beside the XLSX.
#[derive(Debug, Clone)]
pub struct ProjectStore {
    state_directory: PathBuf,
}

impl ProjectStore {
    pub fn new(state_directory: impl Into<PathBuf>) -> Self {
        Self {
            state_directory: state_directory.into(),
        }
    }

    pub fn from_os_app_data() -> Result<Self> {
        let base = os_application_data_directory()?;
        Ok(Self::new(base.join("document-harvester")))
    }

    pub fn state_file_path(&self) -> PathBuf {
        self.state_directory.join(STATE_FILE_NAME)
    }

    pub fn list_projects(&self) -> Result<Vec<ProjectRecord>> {
        Ok(self.load_index()?.projects)
    }

    pub fn current_project(&self) -> Result<Option<ProjectRecord>> {
        let index = self.load_index()?;
        Ok(index.current_project_id.and_then(|current_id| {
            index
                .projects
                .into_iter()
                .find(|project| project.id == current_id)
        }))
    }

    pub fn create_project(
        &self,
        requested_name: &str,
        parent_directory: &Path,
        writer: &XlsxWriter,
    ) -> Result<ProjectRecord> {
        let name = validate_project_name(requested_name)?;
        let parent = fs::canonicalize(parent_directory).map_err(|error| {
            DhError::InvalidInput(format!(
                "cannot access project parent directory '{}': {error}",
                parent_directory.display()
            ))
        })?;
        if !parent.is_dir() {
            return Err(DhError::InvalidInput(format!(
                "project parent is not a directory: '{}'",
                parent.display()
            )));
        }

        // Read and validate private state before creating any project files.
        let mut index = self.load_index()?;
        let project_directory = parent.join(&name);
        fs::create_dir(&project_directory).map_err(|error| {
            DhError::State(format!(
                "cannot create project directory '{}': {error}",
                project_directory.display()
            ))
        })?;

        let workbook_file_name = if name.to_ascii_lowercase().ends_with(".xlsx") {
            name.clone()
        } else {
            format!("{name}.xlsx")
        };
        let workbook_path = project_directory.join(workbook_file_name);

        if let Err(error) = writer.create_blank(&workbook_path) {
            let _ = fs::remove_dir_all(&project_directory);
            return Err(error);
        }

        let created_at_unix_ms = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis();
        let id = format!("p-{created_at_unix_ms}-{}", unique_suffix());

        let record = ProjectRecord {
            id: id.clone(),
            name,
            project_directory: project_directory.clone(),
            workbook_path,
            created_at_unix_ms,
        };

        index.projects.push(record.clone());
        index.current_project_id = Some(id);

        if let Err(error) = self.save_index(&index) {
            let _ = fs::remove_dir_all(&project_directory);
            return Err(error);
        }

        Ok(record)
    }

    fn load_index(&self) -> Result<ProjectIndex> {
        let path = self.state_file_path();
        let bytes = match fs::read(&path) {
            Ok(bytes) => bytes,
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
                return Ok(ProjectIndex::default());
            }
            Err(error) => {
                return Err(DhError::State(format!(
                    "cannot read private project state '{}': {error}",
                    path.display()
                )));
            }
        };

        let index: ProjectIndex = serde_json::from_slice(&bytes).map_err(|error| {
            DhError::State(format!(
                "private project state is invalid '{}': {error}",
                path.display()
            ))
        })?;
        if index.version != STATE_VERSION {
            return Err(DhError::State(format!(
                "unsupported private project-state version {}",
                index.version
            )));
        }
        Ok(index)
    }

    fn save_index(&self, index: &ProjectIndex) -> Result<()> {
        fs::create_dir_all(&self.state_directory).map_err(|error| {
            DhError::State(format!(
                "cannot create private application-data directory '{}': {error}",
                self.state_directory.display()
            ))
        })?;

        let bytes = serde_json::to_vec_pretty(index)
            .map_err(|error| DhError::State(format!("cannot serialize project state: {error}")))?;
        let id = unique_suffix();
        let temporary_path = self
            .state_directory
            .join(format!("{STATE_FILE_NAME}.tmp-{id}"));
        let backup_path = self
            .state_directory
            .join(format!("{STATE_FILE_NAME}.backup-{id}"));

        let mut temporary_file = OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&temporary_path)
            .map_err(|error| {
                DhError::State(format!(
                    "cannot create temporary project state '{}': {error}",
                    temporary_path.display()
                ))
            })?;
        if let Err(error) = temporary_file
            .write_all(&bytes)
            .and_then(|()| temporary_file.sync_all())
        {
            let _ = fs::remove_file(&temporary_path);
            return Err(DhError::State(format!(
                "cannot write temporary project state: {error}"
            )));
        }
        drop(temporary_file);

        let destination = self.state_file_path();
        if destination.exists() {
            fs::rename(&destination, &backup_path).map_err(|error| {
                let _ = fs::remove_file(&temporary_path);
                DhError::State(format!(
                    "cannot preserve previous project state '{}': {error}",
                    destination.display()
                ))
            })?;

            if let Err(error) = fs::rename(&temporary_path, &destination) {
                let _ = fs::rename(&backup_path, &destination);
                let _ = fs::remove_file(&temporary_path);
                return Err(DhError::State(format!(
                    "cannot replace private project state '{}': {error}",
                    destination.display()
                )));
            }
            let _ = fs::remove_file(backup_path);
        } else if let Err(error) = fs::rename(&temporary_path, &destination) {
            let _ = fs::remove_file(&temporary_path);
            return Err(DhError::State(format!(
                "cannot install private project state '{}': {error}",
                destination.display()
            )));
        }

        Ok(())
    }
}

fn validate_project_name(name: &str) -> Result<String> {
    let name = name.trim();
    if name.is_empty() {
        return Err(DhError::InvalidInput(
            "project name cannot be empty".to_string(),
        ));
    }
    if name == "." || name == ".." {
        return Err(DhError::InvalidInput(
            "project name cannot be a path-navigation component".to_string(),
        ));
    }
    if name.chars().any(|character| {
        character.is_control()
            || matches!(
                character,
                '<' | '>' | ':' | '"' | '/' | '\\' | '|' | '?' | '*'
            )
    }) {
        return Err(DhError::InvalidInput(
            "project name contains a character that is not portable across Windows and Linux"
                .to_string(),
        ));
    }
    if name.ends_with('.') || name.ends_with(' ') {
        return Err(DhError::InvalidInput(
            "project name cannot end with a dot or space".to_string(),
        ));
    }

    let device_name = name.split('.').next().unwrap_or(name).to_ascii_uppercase();
    let reserved = matches!(device_name.as_str(), "CON" | "PRN" | "AUX" | "NUL")
        || (device_name.len() == 4
            && (device_name.starts_with("COM") || device_name.starts_with("LPT"))
            && device_name.as_bytes()[3].is_ascii_digit()
            && device_name.as_bytes()[3] != b'0');
    if reserved {
        return Err(DhError::InvalidInput(
            "project name is reserved by Windows".to_string(),
        ));
    }

    Ok(name.to_string())
}

fn os_application_data_directory() -> Result<PathBuf> {
    #[cfg(target_os = "windows")]
    {
        if let Some(path) = std::env::var_os("APPDATA") {
            return Ok(PathBuf::from(path));
        }
        if let Some(home) = std::env::var_os("USERPROFILE") {
            return Ok(PathBuf::from(home).join("AppData").join("Roaming"));
        }
    }

    #[cfg(target_os = "macos")]
    {
        if let Some(home) = std::env::var_os("HOME") {
            return Ok(PathBuf::from(home)
                .join("Library")
                .join("Application Support"));
        }
    }

    #[cfg(target_os = "linux")]
    {
        if let Some(path) = std::env::var_os("XDG_DATA_HOME") {
            return Ok(PathBuf::from(path));
        }
        if let Some(home) = std::env::var_os("HOME") {
            return Ok(PathBuf::from(home).join(".local").join("share"));
        }
    }

    Err(DhError::State(
        "cannot determine the OS application-data directory".to_string(),
    ))
}

fn unique_suffix() -> String {
    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_nanos())
        .unwrap_or_default();
    format!("{timestamp}-{}", std::process::id())
}

#[cfg(test)]
mod tests {
    use super::*;
    use calamine::{open_workbook, Reader, Xlsx};

    fn temporary_directory(label: &str) -> PathBuf {
        let path = std::env::temp_dir().join(format!("{label}-{}", unique_suffix()));
        fs::create_dir_all(&path).expect("create test directory");
        path
    }

    #[test]
    fn creates_workbook_and_keeps_private_state_outside_project_directory() {
        let root = temporary_directory("dh-project-create");
        let parent = root.join("projects");
        let state = root.join("private-app-data");
        fs::create_dir_all(&parent).expect("create project parent");

        let store = ProjectStore::new(&state);
        let record = store
            .create_project("Research 2026", &parent, &XlsxWriter)
            .expect("create project");

        assert!(record.project_directory.is_dir());
        assert!(record.workbook_path.is_file());
        assert!(store.state_file_path().is_file());

        // Independent read-back: verify the actual project workbook.
        let mut independent: Xlsx<_> =
            open_workbook(&record.workbook_path).expect("calamine opens project workbook");
        assert!(independent
            .sheet_names()
            .iter()
            .any(|name| name == "Sheet1"));
        let sheet = independent
            .worksheet_range("Sheet1")
            .expect("default destination worksheet is readable");
        assert_eq!(sheet.used_cells().count(), 0);

        // CI preserves this exact workbook for the LibreOffice import check
        // and as a build-evidence artifact.
        if let Some(destination) = std::env::var_os("DH_S1_WORKBOOK_ARTIFACT") {
            let destination = PathBuf::from(destination);
            if let Some(parent) = destination.parent() {
                fs::create_dir_all(parent).expect("create workbook artifact directory");
            }
            fs::copy(&record.workbook_path, &destination)
                .expect("preserve project-created workbook artifact");
        }
        drop(sheet);
        drop(independent);
        assert!(!record.project_directory.join(STATE_FILE_NAME).exists());
        assert_eq!(fs::read_dir(&record.project_directory).unwrap().count(), 1);

        let reloaded = ProjectStore::new(&state);
        assert_eq!(reloaded.list_projects().unwrap(), vec![record.clone()]);
        assert_eq!(reloaded.current_project().unwrap(), Some(record));

        fs::remove_dir_all(root).expect("remove test directory");
    }

    #[test]
    fn existing_project_is_not_overwritten() {
        let root = temporary_directory("dh-project-duplicate");
        let parent = root.join("projects");
        fs::create_dir_all(&parent).expect("create project parent");
        let store = ProjectStore::new(root.join("private-app-data"));

        let record = store
            .create_project("Existing", &parent, &XlsxWriter)
            .expect("create first project");
        let before = fs::read(&record.workbook_path).expect("read original workbook");

        assert!(store
            .create_project("Existing", &parent, &XlsxWriter)
            .is_err());
        assert_eq!(
            fs::read(&record.workbook_path).expect("read workbook after duplicate"),
            before
        );

        fs::remove_dir_all(root).expect("remove test directory");
    }

    #[test]
    fn rejects_path_traversal_and_nonportable_names() {
        for name in [
            "",
            " ",
            ".",
            "..",
            "../escape",
            "a/b",
            "a\\b",
            "CON",
            "NUL.txt",
        ] {
            assert!(validate_project_name(name).is_err(), "accepted {name:?}");
        }
        assert_eq!(validate_project_name("  Report  ").unwrap(), "Report");
    }

    #[test]
    fn keeps_multiple_projects_and_updates_current_project() {
        let root = temporary_directory("dh-project-history");
        let parent = root.join("projects");
        fs::create_dir_all(&parent).expect("create project parent");
        let state = root.join("private-app-data");
        let store = ProjectStore::new(&state);

        let first = store
            .create_project("First", &parent, &XlsxWriter)
            .expect("create first project");
        let second = store
            .create_project("Second", &parent, &XlsxWriter)
            .expect("create second project");

        let reloaded = ProjectStore::new(&state);
        let projects = reloaded.list_projects().expect("read project history");
        assert_eq!(projects, vec![first, second.clone()]);
        assert_eq!(reloaded.current_project().unwrap(), Some(second));

        fs::remove_dir_all(root).expect("remove test directory");
    }

    #[test]
    fn missing_private_state_is_an_empty_store_and_does_not_create_files() {
        let root = temporary_directory("dh-project-missing-state");
        let state = root.join("private-app-data");
        let store = ProjectStore::new(&state);

        assert!(store.list_projects().unwrap().is_empty());
        assert_eq!(store.current_project().unwrap(), None);
        assert!(!state.exists());

        fs::remove_dir_all(root).expect("remove test directory");
    }

    #[test]
    fn corrupt_private_state_fails_closed_and_is_not_overwritten() {
        let root = temporary_directory("dh-project-corrupt-state");
        let parent = root.join("projects");
        let state = root.join("private-app-data");
        fs::create_dir_all(&parent).expect("create project parent");
        fs::create_dir_all(&state).expect("create private state directory");
        let original = b"{ definitely not valid project state";
        fs::write(state.join(STATE_FILE_NAME), original).expect("write corrupt state");

        let store = ProjectStore::new(&state);
        assert!(store.list_projects().is_err());
        assert!(store.current_project().is_err());
        assert!(store
            .create_project("Must Not Be Created", &parent, &XlsxWriter)
            .is_err());
        assert_eq!(
            fs::read(state.join(STATE_FILE_NAME)).unwrap(),
            original,
            "corrupt private state must not be replaced implicitly"
        );
        assert!(!parent.join("Must Not Be Created").exists());

        fs::remove_dir_all(root).expect("remove test directory");
    }

    #[test]
    fn invalid_workbook_parent_path_fails_without_registering_a_project() {
        let root = temporary_directory("dh-project-workbook-path-failure");
        let not_a_directory = root.join("not-a-directory");
        fs::write(&not_a_directory, b"file").expect("create non-directory parent");
        let state = root.join("private-app-data");
        let store = ProjectStore::new(&state);

        assert!(store
            .create_project("Cannot Be Created", &not_a_directory, &XlsxWriter)
            .is_err());
        assert!(store.list_projects().unwrap().is_empty());
        assert!(!state.exists());

        fs::remove_dir_all(root).expect("remove test directory");
    }

    #[test]
    fn project_workbooks_remain_isolated_after_store_restart() {
        let root = temporary_directory("dh-project-isolation");
        let parent = root.join("projects");
        let state = root.join("private-app-data");
        fs::create_dir_all(&parent).expect("create project parent");
        let store = ProjectStore::new(&state);

        let alpha = store
            .create_project("Alpha", &parent, &XlsxWriter)
            .expect("create Alpha");
        let beta = store
            .create_project("Beta", &parent, &XlsxWriter)
            .expect("create Beta");

        assert_ne!(alpha.id, beta.id);
        assert_ne!(alpha.workbook_path, beta.workbook_path);
        assert_ne!(alpha.project_directory, beta.project_directory);
        assert!(alpha.workbook_path.is_file());
        assert!(beta.workbook_path.is_file());

        let restarted_store = ProjectStore::new(&state);
        let projects = restarted_store.list_projects().unwrap();
        assert_eq!(projects, vec![alpha.clone(), beta.clone()]);
        assert_eq!(
            restarted_store.current_project().unwrap(),
            Some(beta.clone())
        );

        assert!(!alpha.project_directory.join(STATE_FILE_NAME).exists());
        assert!(!alpha.project_directory.join("projects.json").exists());
        assert!(!beta.project_directory.join(STATE_FILE_NAME).exists());
        assert!(!beta.project_directory.join("projects.json").exists());

        fs::remove_dir_all(root).expect("remove test directory");
    }
}
