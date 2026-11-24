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
const STATE_VERSION: u32 = 2;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ProjectRecord {
    pub id: String,
    pub name: String,
    pub project_directory: PathBuf,
    pub workbook_path: PathBuf,
    pub created_at_unix_ms: u128,
    /// Path of the PDF copy inside private app-data, used by the viewer after import.
    /// The original source path is deliberately not retained.
    #[serde(default)]
    pub imported_pdf_path: Option<PathBuf>,
    /// Committed capture records in commit order; the workbook is rebuilt from
    /// this list. Empty until the first capture is approved.
    #[serde(default)]
    pub captures: Vec<dh_writer::CaptureRecord>,
    /// SHA-256 hex digest of the last committed workbook; `None` until the
    /// first commit.
    #[serde(default)]
    pub workbook_sha256: Option<String>,
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
            imported_pdf_path: None,
            captures: Vec::new(),
            workbook_sha256: None,
        };

        index.projects.push(record.clone());
        index.current_project_id = Some(id);

        if let Err(error) = self.save_index(&index) {
            let _ = fs::remove_dir_all(&project_directory);
            return Err(error);
        }

        Ok(record)
    }

    /// Record a committed capture list and the committed workbook hash.
    ///
    /// The capture list is the authoritative content of the project workbook;
    /// the workbook hash lets the writer refuse rebuilds over externally
    /// modified files. State shape changed with this task, hence the state
    /// version bump to 2.
    pub fn commit_capture_list(
        &self,
        project_id: &str,
        captures: &[dh_writer::CaptureRecord],
        workbook_sha256: String,
    ) -> Result<()> {
        let mut index = self.load_index()?;
        let project = index
            .projects
            .iter_mut()
            .find(|project| project.id == project_id)
            .ok_or_else(|| {
                DhError::State(format!(
                    "project '{}' is not present in private state",
                    project_id
                ))
            })?;
        project.captures = captures.to_vec();
        project.workbook_sha256 = Some(workbook_sha256);
        self.save_index(&index)
    }

    /// Read the committed capture list of a project.
    pub fn capture_list(&self, project_id: &str) -> Result<Vec<dh_writer::CaptureRecord>> {
        let index = self.load_index()?;
        let project = index
            .projects
            .iter()
            .find(|project| project.id == project_id)
            .ok_or_else(|| {
                DhError::State(format!(
                    "project '{}' is not present in private state",
                    project_id
                ))
            })?;
        Ok(project.captures.clone())
    }

    /// Read the hash of the last committed workbook, if any.
    pub fn committed_workbook_sha256(&self, project_id: &str) -> Result<Option<String>> {
        let index = self.load_index()?;
        let project = index
            .projects
            .iter()
            .find(|project| project.id == project_id)
            .ok_or_else(|| {
                DhError::State(format!(
                    "project '{}' is not present in private state",
                    project_id
                ))
            })?;
        Ok(project.workbook_sha256.clone())
    }

    /// Transaction journal path for one project inside private application data.
    pub fn journal_path(&self, project_id: &str) -> PathBuf {
        self.state_directory
            .join("projects")
            .join(project_id)
            .join("transaction.journal.jsonl")
    }

    /// Recover private state from a journaled capture-list payload.
    ///
    /// The journal stores the full capture list before the workbook write
    /// begins, so a crash after the workbook replacement but before
    /// `commit_capture_list` can be repaired on restart. When the on-disk
    /// workbook matches a journal entry with a payload, the private capture
    /// list and workbook hash are written from that payload. Returns `None`
    /// when the journal has nothing to restore or private state is already
    /// current. A payload version this build does not understand fails closed.
    pub fn recover_pending_transaction(&self, project_id: &str) -> Result<Option<JournalRecovery>> {
        let index = self.load_index()?;
        let project = index
            .projects
            .iter()
            .find(|project| project.id == project_id)
            .ok_or_else(|| {
                DhError::State(format!(
                    "project '{}' is not present in private state",
                    project_id
                ))
            })?;
        let journal = dh_writer::TransactionJournal::new(&self.journal_path(project_id));
        let Some(resolution) = journal.recover(&project.workbook_path)? else {
            return Ok(None);
        };
        if resolution.outcome != dh_writer::JournalOutcome::Committed {
            return Ok(None);
        }
        let Some(payload) = resolution.entry.payload.clone() else {
            return Ok(None);
        };
        if payload.version != dh_writer::JOURNAL_PAYLOAD_VERSION {
            return Err(DhError::State(format!(
                "transaction journal payload version {} is not supported",
                payload.version
            )));
        }
        let Some(workbook_sha256) = resolution.entry.workbook_sha256_after_commit.clone() else {
            return Err(DhError::State(
                "transaction journal payload has no committed workbook hash".to_string(),
            ));
        };
        if project.captures == payload.captures
            && project.workbook_sha256.as_deref() == Some(workbook_sha256.as_str())
        {
            return Ok(None);
        }
        self.commit_capture_list(project_id, &payload.captures, workbook_sha256.clone())?;
        Ok(Some(JournalRecovery {
            captures: payload.captures,
            workbook_sha256,
        }))
    }

    /// Committed capture history with destination row ranges.
    ///
    /// The destination range is the half-open workbook row range `[start, end)`
    /// allocated to the capture; captures append below the previous one with no
    /// separator row.
    pub fn capture_history(&self, project_id: &str) -> Result<Vec<CaptureHistoryEntry>> {
        let captures = self.capture_list(project_id)?;
        let spans = dh_writer::capture_row_spans(&captures);
        Ok(captures
            .into_iter()
            .enumerate()
            .map(|(index, capture)| CaptureHistoryEntry {
                capture,
                destination_start_row: spans.get(index).map(|(start, _end)| *start).unwrap_or(0),
                destination_end_row: spans.get(index).map(|(_start, end)| *end).unwrap_or(0),
            })
            .collect())
    }

    /// Undo the latest committed capture and persist the reduced capture list.
    ///
    /// The undo rebuilds the workbook from the remaining captures through the
    /// same writer and verification path and refuses unsafe states.
    pub fn undo_last_capture(
        &self,
        project_id: &str,
        writer: &XlsxWriter,
    ) -> Result<dh_writer::CaptureRecord> {
        let captures = self.capture_list(project_id)?;
        if captures.is_empty() {
            return Err(DhError::State("no committed captures to undo".to_string()));
        }
        let removed = captures[captures.len() - 1].clone();
        let remaining = captures[..captures.len() - 1].to_vec();
        let record = self
            .load_index()?
            .projects
            .into_iter()
            .find(|project| project.id == project_id)
            .ok_or_else(|| {
                DhError::State(format!(
                    "project '{}' is not present in private state",
                    project_id
                ))
            })?;
        let committed_hash = self
            .committed_workbook_sha256(project_id)?
            .unwrap_or_default();
        let journal = dh_writer::TransactionJournal::new(&self.journal_path(project_id));
        let new_hash = writer.undo_capture_list(
            &captures,
            &record.workbook_path,
            &committed_hash,
            &journal,
            None,
            None,
        )?;
        self.commit_capture_list(project_id, &remaining, new_hash)?;
        Ok(removed)
    }

    /// Copy a source PDF into private application data and remember the copy.
    ///
    /// The original path is used only for the duration of this call. All later
    /// viewer work must use the returned path, read back from project state. A
    /// failed copy leaves the state and any earlier import untouched.
    pub fn import_pdf(&self, project_id: &str, source_pdf: &Path) -> Result<PathBuf> {
        let mut index = self.load_index()?;
        let project = index
            .projects
            .iter_mut()
            .find(|project| project.id == project_id)
            .ok_or_else(|| {
                DhError::InvalidInput(format!("no private project record for '{project_id}'"))
            })?;

        let source = fs::canonicalize(source_pdf).map_err(|error| {
            DhError::InvalidInput(format!(
                "cannot access source PDF '{}': {error}",
                source_pdf.display()
            ))
        })?;
        if !source.is_file() {
            return Err(DhError::InvalidInput(format!(
                "source PDF is not a file: '{}'",
                source.display()
            )));
        }

        let expected_length = fs::metadata(&source)
            .map_err(|error| {
                DhError::State(format!(
                    "cannot inspect source PDF '{}': {error}",
                    source.display()
                ))
            })?
            .len();
        let directory = self.state_directory.join("projects").join(project_id);
        fs::create_dir_all(&directory).map_err(|error| {
            DhError::State(format!(
                "cannot create private PDF directory '{}': {error}",
                directory.display()
            ))
        })?;
        let destination = directory.join(format!("source-{}.pdf", unique_suffix()));
        let copied_length = fs::copy(&source, &destination).map_err(|error| {
            let _ = fs::remove_file(&destination);
            DhError::State(format!("cannot copy source PDF into app data: {error}"))
        })?;
        if copied_length != expected_length {
            let _ = fs::remove_file(&destination);
            return Err(DhError::State(
                "the copied PDF has a different length than the source".to_owned(),
            ));
        }

        let previous = project.imported_pdf_path.replace(destination.clone());
        if let Err(error) = self.save_index(&index) {
            let _ = fs::remove_file(&destination);
            return Err(error);
        }
        let private_projects = self.state_directory.join("projects");
        if let Some(previous) = previous {
            if previous != destination && previous.starts_with(&private_projects) {
                let _ = fs::remove_file(previous);
            }
        }
        Ok(destination)
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

/// One committed capture with its allocated destination row range in the
/// workbook.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CaptureHistoryEntry {
    pub capture: dh_writer::CaptureRecord,
    pub destination_start_row: u32,
    pub destination_end_row: u32,
}

/// Private-state repair applied from a journaled capture-list payload.
#[derive(Debug, Clone, PartialEq)]
pub struct JournalRecovery {
    pub captures: Vec<dh_writer::CaptureRecord>,
    pub workbook_sha256: String,
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

    fn owner_pdf(name: &str) -> PathBuf {
        Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("..")
            .join("..")
            .join("PDF_tests")
            .join(name)
    }

    #[test]
    fn import_copies_the_pdf_into_private_app_data_and_records_only_the_copy() {
        let root = temporary_directory("dh-project-import");
        let parent = root.join("projects");
        let state = root.join("private-app-data");
        fs::create_dir_all(&parent).expect("create project parent");
        let store = ProjectStore::new(&state);
        let record = store
            .create_project("Imported", &parent, &XlsxWriter)
            .expect("create project");
        let original = owner_pdf("01.pdf");
        let original_bytes = fs::read(&original).expect("read owner PDF 01.pdf");

        let copy = store
            .import_pdf(&record.id, &original)
            .expect("copy owner PDF into app data");

        // Independent oracle: compare raw bytes, not code-under-test state.
        assert_eq!(
            fs::read(&copy).expect("read stored copy"),
            original_bytes,
            "the stored copy must be byte-identical to the owner PDF"
        );
        assert!(copy.starts_with(state.join("projects")));
        assert_ne!(copy, original);
        let restored = store
            .current_project()
            .expect("read state")
            .expect("project");
        assert_eq!(restored.imported_pdf_path.as_deref(), Some(copy.as_path()));
        assert!(!restored.project_directory.join("01.pdf").exists());
        assert!(original.is_file(), "the original must not be moved");
        assert_eq!(
            fs::read(&original).expect("re-read owner PDF"),
            original_bytes,
            "the owner fixture must not be modified"
        );

        fs::remove_dir_all(root).expect("remove test directory");
    }

    #[test]
    fn copy_failure_leaves_the_project_state_unchanged() {
        let root = temporary_directory("dh-project-import-failure");
        let parent = root.join("projects");
        let state = root.join("private-app-data");
        fs::create_dir_all(&parent).expect("create project parent");
        let store = ProjectStore::new(&state);
        let record = store
            .create_project("Unchanged", &parent, &XlsxWriter)
            .expect("create project");

        let error = store
            .import_pdf(&record.id, &owner_pdf("missing.pdf"))
            .expect_err("missing owner PDF must fail");
        assert!(error.to_string().contains("cannot access source PDF"));

        let restored = store
            .current_project()
            .expect("read state")
            .expect("project");
        assert_eq!(restored.imported_pdf_path, None);
        assert!(!state.join("projects").join(&record.id).exists());

        fs::remove_dir_all(root).expect("remove test directory");
    }

    fn text_cell(text: &str) -> dh_ir::Cell {
        dh_ir::Cell {
            state: dh_ir::CellState::Text,
            text: Some(text.to_string()),
            numeric: None,
            raw: Some(text.to_string()),
            raw_run_texts: vec![text.to_string()],
            bbox: None,
            grid_position: Some((0, 0)),
            rowspan: 1,
            colspan: 1,
            covered_by: None,
            fonts: Vec::new(),
            raw_confidence: None,
            calibrated_confidence: None,
            guard: dh_ir::GuardStatus::Pass,
            provenance: dh_ir::Provenance {
                page: 1,
                region: dh_ir::BBox {
                    x0: 0.0,
                    y0: 0.0,
                    x1: 100.0,
                    y1: 50.0,
                },
                extractor: "scaffold".to_string(),
                extractor_version: "scaffold".to_string(),
                run_ids: Vec::new(),
                run_order: Vec::new(),
                normalization_ops: Vec::new(),
                run_join_policy_version: "scaffold".to_string(),
                cell_serialization_policy_version: "scaffold".to_string(),
                numeric_policy_version: "scaffold".to_string(),
            },
        }
    }

    fn capture_with_rows(sequence: u64, rows: &[&str]) -> dh_writer::CaptureRecord {
        let rows = rows
            .iter()
            .enumerate()
            .map(|(row_index, text)| {
                let mut cell = text_cell(text);
                cell.grid_position = Some((row_index as u32, 0));
                vec![cell]
            })
            .collect::<Vec<_>>();
        dh_writer::CaptureRecord {
            id: format!("capture-{sequence}"),
            source_pdf_sha256: format!("source-sha-{sequence}"),
            page: 2,
            region: dh_ir::BBox {
                x0: 10.0,
                y0: 20.0,
                x1: 110.0,
                y1: 70.0,
            },
            commit_sequence: sequence,
            blocks: vec![dh_ir::TableBlock {
                page: 2,
                bbox: dh_ir::BBox {
                    x0: 10.0,
                    y0: 20.0,
                    x1: 110.0,
                    y1: 70.0,
                },
                rows,
                direction: None,
                segmentation_confidence: None,
                continuation: None,
            }],
        }
    }

    /// Commit one capture list through the production writer and private state.
    fn commit(
        store: &ProjectStore,
        record: &ProjectRecord,
        captures: &[dh_writer::CaptureRecord],
    ) -> String {
        let committed_hash = store
            .committed_workbook_sha256(&record.id)
            .expect("read committed hash")
            .unwrap_or_default();
        let journal = dh_writer::TransactionJournal::new(&store.journal_path(&record.id));
        let new_hash = XlsxWriter
            .write_capture_list_with_journal(
                captures,
                &record.workbook_path,
                &committed_hash,
                &journal,
                None,
                None,
            )
            .expect("commit capture list");
        store
            .commit_capture_list(&record.id, captures, new_hash.clone())
            .expect("record committed capture list");
        new_hash
    }

    /// Run the writer transaction without the following private-state write,
    /// simulating a crash between workbook replacement and state persistence.
    fn write_without_state(
        store: &ProjectStore,
        record: &ProjectRecord,
        captures: &[dh_writer::CaptureRecord],
        failure_stage: dh_writer::FailureStage,
    ) -> dh_core::DhError {
        let committed_hash = store
            .committed_workbook_sha256(&record.id)
            .expect("read committed hash")
            .unwrap_or_default();
        let journal = dh_writer::TransactionJournal::new(&store.journal_path(&record.id));
        XlsxWriter
            .write_capture_list_with_journal(
                captures,
                &record.workbook_path,
                &committed_hash,
                &journal,
                Some(failure_stage),
                None,
            )
            .expect_err("injected failure must abort the transaction")
    }

    fn undo_without_state(
        store: &ProjectStore,
        record: &ProjectRecord,
        captures: &[dh_writer::CaptureRecord],
    ) -> dh_core::DhError {
        let committed_hash = store
            .committed_workbook_sha256(&record.id)
            .expect("read committed hash")
            .unwrap_or_default();
        let journal = dh_writer::TransactionJournal::new(&store.journal_path(&record.id));
        XlsxWriter
            .undo_capture_list(
                captures,
                &record.workbook_path,
                &committed_hash,
                &journal,
                Some(dh_writer::FailureStage::BeforeJournalCompletion),
                None,
            )
            .expect_err("injected undo failure must abort the transaction")
    }

    /// Raw OOXML oracle: concatenated XML entries from the workbook package.
    fn workbook_xml(path: &Path) -> String {
        use std::io::Read;

        let file = fs::File::open(path).expect("open workbook package");
        let mut archive = zip::ZipArchive::new(file).expect("read OOXML zip");
        let names: Vec<String> = archive
            .file_names()
            .filter_map(|name| name.ok())
            .filter(|name| name.ends_with(".xml"))
            .map(|name| name.to_string())
            .collect();
        let mut corpus = String::new();
        for name in names {
            let mut entry = archive.by_name(&name).expect("read OOXML entry");
            let mut buffer = Vec::new();
            entry.read_to_end(&mut buffer).expect("read OOXML bytes");
            corpus.push_str(&String::from_utf8_lossy(&buffer));
            corpus.push('\n');
        }
        corpus
    }

    fn cell_text(path: &Path, row: u32, column: u32) -> Option<String> {
        let mut workbook: Xlsx<_> = open_workbook(path).expect("calamine opens workbook");
        let sheet = workbook
            .worksheet_range("Sheet1")
            .expect("read destination worksheet");
        match sheet.get_value((row, column)) {
            Some(calamine::Data::String(text)) => Some(text.clone()),
            _ => None,
        }
    }

    #[test]
    fn capture_history_lists_committed_captures_in_order_with_destination_ranges() {
        let root = temporary_directory("dh-project-history-ranges");
        let parent = root.join("projects");
        fs::create_dir_all(&parent).expect("create project parent");
        let store = ProjectStore::new(root.join("private-app-data"));
        let record = store
            .create_project("History", &parent, &XlsxWriter)
            .expect("create project");

        assert!(
            store
                .capture_history(&record.id)
                .expect("empty history")
                .is_empty(),
            "a new project has no committed captures"
        );

        let first = capture_with_rows(1, &["Name", "Age"]);
        let second = capture_with_rows(2, &["Item"]);
        let captures = vec![first.clone(), second.clone()];
        commit(&store, &record, &captures);

        let history = store
            .capture_history(&record.id)
            .expect("read capture history");
        assert_eq!(history.len(), 2, "one history entry per committed capture");
        assert_eq!(history[0].capture, first, "history keeps commit order");
        assert_eq!(history[1].capture, second, "history keeps commit order");
        assert_eq!(
            (
                history[0].destination_start_row,
                history[0].destination_end_row
            ),
            (0, 2),
            "first capture occupies rows 0..2"
        );
        assert_eq!(
            (
                history[1].destination_start_row,
                history[1].destination_end_row
            ),
            (2, 3),
            "second capture appends directly below with no separator row"
        );
        assert_eq!(history[1].capture.source_pdf_sha256, "source-sha-2");
        assert_eq!(history[1].capture.page, 2);
        assert_eq!(history[1].capture.region.x0, 10.0);

        // Independent oracle: the reported destination range starts at the row
        // where the capture's first text is actually written.
        let mut workbook: Xlsx<_> =
            open_workbook(&record.workbook_path).expect("calamine opens workbook");
        let sheet = workbook
            .worksheet_range("Sheet1")
            .expect("read destination worksheet");
        for entry in &history {
            assert_eq!(
                sheet.get_value((entry.destination_start_row, 0)),
                Some(&calamine::Data::String(
                    entry.capture.blocks[0].rows[0][0]
                        .text
                        .clone()
                        .expect("capture text")
                )),
                "history destination start row must match the written capture"
            );
        }

        fs::remove_dir_all(root).expect("remove test directory");
    }

    #[test]
    fn undo_last_capture_restores_previous_workbook_and_private_state() {
        let root = temporary_directory("dh-project-undo");
        let parent = root.join("projects");
        fs::create_dir_all(&parent).expect("create project parent");
        let store = ProjectStore::new(root.join("private-app-data"));
        let record = store
            .create_project("Undo", &parent, &XlsxWriter)
            .expect("create project");

        let capture_a = capture_with_rows(1, &["Alpha"]);
        let capture_b = capture_with_rows(2, &["Beta", "Beta row 2"]);
        commit(&store, &record, std::slice::from_ref(&capture_a));
        commit(&store, &record, &[capture_a.clone(), capture_b.clone()]);

        let removed = store
            .undo_last_capture(&record.id, &XlsxWriter)
            .expect("undo latest capture");
        assert_eq!(removed, capture_b, "undo removes only the latest capture");
        assert_eq!(
            store.capture_list(&record.id).expect("read captures"),
            vec![capture_a.clone()],
            "private state keeps the earlier captures"
        );
        let history = store
            .capture_history(&record.id)
            .expect("read history after undo");
        assert_eq!(history.len(), 1);
        assert_eq!(history[0].capture, capture_a);

        let mut workbook: Xlsx<_> =
            open_workbook(&record.workbook_path).expect("calamine opens workbook");
        let sheet = workbook
            .worksheet_range("Sheet1")
            .expect("read destination worksheet");
        assert_eq!(
            sheet.get_value((0, 0)),
            Some(&calamine::Data::String("Alpha".to_string())),
            "undo restores the workbook built from the remaining captures"
        );
        assert_eq!(
            sheet.get_value((1, 0)),
            None,
            "the undone capture must not remain in the workbook"
        );

        let journal = dh_writer::TransactionJournal::new(&store.journal_path(&record.id));
        let entries = journal.load().expect("load transaction journal");
        let undo_entry = entries.last().expect("undo journal entry exists");
        assert_eq!(
            undo_entry.undone_commit_sequence,
            Some(capture_b.commit_sequence),
            "the journal records which capture was undone"
        );

        fs::remove_dir_all(root).expect("remove test directory");
    }

    #[test]
    fn undo_with_empty_history_is_refused_without_touching_state() {
        let root = temporary_directory("dh-project-undo-empty");
        let parent = root.join("projects");
        fs::create_dir_all(&parent).expect("create project parent");
        let store = ProjectStore::new(root.join("private-app-data"));
        let record = store
            .create_project("Undo Empty", &parent, &XlsxWriter)
            .expect("create project");
        let before = fs::read(&record.workbook_path).expect("read blank workbook");

        let error = store
            .undo_last_capture(&record.id, &XlsxWriter)
            .expect_err("undo with no committed captures must be refused");
        assert!(
            matches!(error, DhError::State(_)),
            "unexpected error: {error}"
        );
        assert_eq!(
            fs::read(&record.workbook_path).expect("re-read workbook"),
            before,
            "refused undo must not modify the workbook"
        );
        assert!(store.capture_list(&record.id).unwrap().is_empty());

        fs::remove_dir_all(root).expect("remove test directory");
    }

    #[test]
    fn undo_targets_the_requested_project_not_the_current_one() {
        let root = temporary_directory("dh-project-undo-target");
        let parent = root.join("projects");
        fs::create_dir_all(&parent).expect("create project parent");
        let store = ProjectStore::new(root.join("private-app-data"));
        let first = store
            .create_project("First", &parent, &XlsxWriter)
            .expect("create first project");
        let second = store
            .create_project("Second", &parent, &XlsxWriter)
            .expect("create second project");
        assert_eq!(
            store.current_project().unwrap().map(|record| record.id),
            Some(second.id.clone())
        );

        let capture = capture_with_rows(1, &["First capture"]);
        commit(&store, &first, std::slice::from_ref(&capture));
        let second_capture = capture_with_rows(1, &["Second capture"]);
        commit(&store, &second, std::slice::from_ref(&second_capture));
        let second_before = fs::read(&second.workbook_path).expect("read second workbook");

        store
            .undo_last_capture(&first.id, &XlsxWriter)
            .expect("undo the requested project's capture");

        assert!(store.capture_list(&first.id).unwrap().is_empty());
        assert_eq!(
            store.capture_list(&second.id).unwrap(),
            vec![second_capture],
            "undo of one project must not touch the other project's state"
        );
        assert_eq!(
            fs::read(&second.workbook_path).expect("re-read second workbook"),
            second_before,
            "undo of one project must not touch the other project's workbook"
        );

        fs::remove_dir_all(root).expect("remove test directory");
    }

    #[test]
    fn crash_after_workbook_replacement_restores_private_state_from_journal_payload() {
        let root = temporary_directory("dh-project-recover-after-replace");
        let parent = root.join("projects");
        fs::create_dir_all(&parent).expect("create project parent");
        let store = ProjectStore::new(root.join("private-app-data"));
        let record = store
            .create_project("Recover", &parent, &XlsxWriter)
            .expect("create project");
        let capture_a = capture_with_rows(1, &["Alpha"]);
        let capture_b = capture_with_rows(2, &["Beta"]);
        commit(&store, &record, std::slice::from_ref(&capture_a));

        let error = write_without_state(
            &store,
            &record,
            &[capture_a.clone(), capture_b.clone()],
            dh_writer::FailureStage::BeforeJournalCompletion,
        );
        assert!(
            matches!(error, DhError::Write(_)),
            "unexpected error: {error}"
        );
        assert_eq!(
            store.capture_list(&record.id).unwrap(),
            vec![capture_a.clone()],
            "the crash leaves private state at the previous capture"
        );

        let recovery = store
            .recover_pending_transaction(&record.id)
            .expect("recover private state")
            .expect("journal payload repairs the missing state write");
        assert_eq!(
            recovery.captures,
            vec![capture_a.clone(), capture_b.clone()]
        );
        assert_eq!(recovery.workbook_sha256.len(), 64);
        assert_eq!(
            store.capture_list(&record.id).unwrap(),
            vec![capture_a, capture_b.clone()],
            "private state is restored from the journal payload"
        );
        assert_eq!(
            store.committed_workbook_sha256(&record.id).unwrap(),
            Some(recovery.workbook_sha256.clone())
        );

        // Independent oracle: calamine sees the capture in the workbook and the
        // history range points at its actual row.
        assert_eq!(
            cell_text(&record.workbook_path, 0, 0),
            Some("Alpha".to_string())
        );
        assert_eq!(
            cell_text(&record.workbook_path, 1, 0),
            Some("Beta".to_string())
        );
        let history = store.capture_history(&record.id).expect("read history");
        assert_eq!(history.len(), 2);
        assert_eq!(
            (
                history[1].destination_start_row,
                history[1].destination_end_row
            ),
            (1, 2),
            "history matches the workbook row occupied by the recovered capture"
        );

        // Raw OOXML oracle: no formula XML and no duplicated text.
        let xml = workbook_xml(&record.workbook_path);
        assert!(
            !xml.contains("<f>"),
            "text must never be written as a formula"
        );
        assert_eq!(
            xml.matches(">Beta<").count(),
            1,
            "recovered capture is not duplicated"
        );

        assert!(
            store
                .recover_pending_transaction(&record.id)
                .expect("second recovery")
                .is_none(),
            "a second recovery must be a no-op"
        );
        assert_eq!(
            store
                .capture_history(&record.id)
                .expect("history after replay")
                .len(),
            2,
            "replay must not duplicate the recovered capture"
        );

        fs::remove_dir_all(root).expect("remove test directory");
    }

    #[test]
    fn crash_before_workbook_write_leaves_private_state_unchanged() {
        let root = temporary_directory("dh-project-recover-before-write");
        let parent = root.join("projects");
        fs::create_dir_all(&parent).expect("create project parent");
        let store = ProjectStore::new(root.join("private-app-data"));
        let record = store
            .create_project("Before Write", &parent, &XlsxWriter)
            .expect("create project");
        let capture_a = capture_with_rows(1, &["Alpha"]);
        let capture_b = capture_with_rows(2, &["Beta"]);
        commit(&store, &record, std::slice::from_ref(&capture_a));
        let workbook_before = fs::read(&record.workbook_path).expect("read workbook");

        for stage in [
            dh_writer::FailureStage::BeforeTempWrite,
            dh_writer::FailureStage::AfterTempWrite,
        ] {
            let error = write_without_state(
                &store,
                &record,
                &[capture_a.clone(), capture_b.clone()],
                stage,
            );
            assert!(
                matches!(error, DhError::Write(_)),
                "unexpected error: {error}"
            );
            assert_eq!(
                fs::read(&record.workbook_path).expect("re-read workbook"),
                workbook_before,
                "a rolled back transaction must not change the workbook"
            );
            assert_eq!(
                store.capture_list(&record.id).unwrap(),
                vec![capture_a.clone()],
                "a rolled back transaction must not change private state"
            );
            assert!(
                store
                    .recover_pending_transaction(&record.id)
                    .expect("recover after rollback")
                    .is_none(),
                "there is nothing to restore for a rolled back transaction"
            );
        }

        assert_eq!(
            cell_text(&record.workbook_path, 0, 0),
            Some("Alpha".to_string())
        );
        assert_eq!(cell_text(&record.workbook_path, 1, 0), None);

        fs::remove_dir_all(root).expect("remove test directory");
    }

    #[test]
    fn undo_crash_after_replacement_reconstructs_reduced_history_from_payload() {
        let root = temporary_directory("dh-project-recover-undo");
        let parent = root.join("projects");
        fs::create_dir_all(&parent).expect("create project parent");
        let store = ProjectStore::new(root.join("private-app-data"));
        let record = store
            .create_project("Undo Recover", &parent, &XlsxWriter)
            .expect("create project");
        let capture_a = capture_with_rows(1, &["Alpha"]);
        let capture_b = capture_with_rows(2, &["Beta"]);
        commit(&store, &record, std::slice::from_ref(&capture_a));
        commit(&store, &record, &[capture_a.clone(), capture_b.clone()]);
        assert_eq!(
            cell_text(&record.workbook_path, 1, 0),
            Some("Beta".to_string())
        );

        let error = undo_without_state(&store, &record, &[capture_a.clone(), capture_b.clone()]);
        assert!(
            matches!(error, DhError::Write(_)),
            "unexpected error: {error}"
        );
        assert_eq!(
            store.capture_list(&record.id).unwrap(),
            vec![capture_a.clone(), capture_b],
            "the crash leaves the pre-undo capture list in private state"
        );
        assert_eq!(
            cell_text(&record.workbook_path, 1, 0),
            None,
            "the workbook was already rebuilt without the undone capture"
        );

        let recovery = store
            .recover_pending_transaction(&record.id)
            .expect("recover undo state")
            .expect("undo payload repairs private state");
        assert_eq!(recovery.captures, vec![capture_a.clone()]);
        assert_eq!(
            store.capture_list(&record.id).unwrap(),
            vec![capture_a.clone()],
            "restart reconstructs the reduced history from the payload"
        );
        let history = store.capture_history(&record.id).expect("read history");
        assert_eq!(history.len(), 1);
        assert_eq!(history[0].capture, capture_a);
        assert_eq!(
            (
                history[0].destination_start_row,
                history[0].destination_end_row
            ),
            (0, 1)
        );

        let xml = workbook_xml(&record.workbook_path);
        assert_eq!(
            xml.matches(">Beta<").count(),
            0,
            "the undone capture is gone"
        );
        assert_eq!(xml.matches(">Alpha<").count(), 1);

        fs::remove_dir_all(root).expect("remove test directory");
    }

    #[test]
    fn replay_of_a_committed_journal_entry_does_not_duplicate_a_capture() {
        let root = temporary_directory("dh-project-replay");
        let parent = root.join("projects");
        fs::create_dir_all(&parent).expect("create project parent");
        let store = ProjectStore::new(root.join("private-app-data"));
        let record = store
            .create_project("Replay", &parent, &XlsxWriter)
            .expect("create project");
        let capture_a = capture_with_rows(1, &["Alpha"]);
        let committed_hash = commit(&store, &record, std::slice::from_ref(&capture_a));

        let journal = dh_writer::TransactionJournal::new(&store.journal_path(&record.id));
        XlsxWriter
            .write_capture_list_with_journal(
                std::slice::from_ref(&capture_a),
                &record.workbook_path,
                &committed_hash,
                &journal,
                None,
                None,
            )
            .expect("replaying the committed capture list is idempotent");
        assert!(
            store
                .recover_pending_transaction(&record.id)
                .expect("recover after replay")
                .is_none(),
            "replaying a committed entry has nothing left to restore"
        );
        assert_eq!(
            store.capture_list(&record.id).unwrap(),
            vec![capture_a],
            "replay does not duplicate the committed capture"
        );
        assert_eq!(store.capture_history(&record.id).unwrap().len(), 1);

        let xml = workbook_xml(&record.workbook_path);
        assert_eq!(
            xml.matches(">Alpha<").count(),
            1,
            "the workbook holds the capture exactly once"
        );

        fs::remove_dir_all(root).expect("remove test directory");
    }

    #[test]
    fn reimport_replaces_only_the_private_copy() {
        let root = temporary_directory("dh-project-reimport");
        let parent = root.join("projects");
        let state = root.join("private-app-data");
        fs::create_dir_all(&parent).expect("create project parent");
        let store = ProjectStore::new(&state);
        let record = store
            .create_project("Reimport", &parent, &XlsxWriter)
            .expect("create project");
        let first = owner_pdf("02.pdf");
        let second = owner_pdf("03.pdf");
        let first_bytes = fs::read(&first).expect("read owner PDF 02.pdf");
        let second_bytes = fs::read(&second).expect("read owner PDF 03.pdf");

        let first_copy = store.import_pdf(&record.id, &first).expect("first import");
        let second_copy = store
            .import_pdf(&record.id, &second)
            .expect("second import");

        assert_ne!(first_copy, second_copy);
        assert!(
            !first_copy.exists(),
            "the superseded private copy is removed"
        );
        assert_eq!(
            fs::read(&second_copy).expect("read second copy"),
            second_bytes,
            "the second stored copy must be byte-identical to owner PDF 03.pdf"
        );
        assert_eq!(
            fs::read(&first).expect("first original untouched"),
            first_bytes,
            "the owner fixture 02.pdf must not be modified"
        );
        assert_eq!(
            fs::read(&second).expect("second original untouched"),
            second_bytes,
            "the owner fixture 03.pdf must not be modified"
        );
        assert_eq!(
            store
                .current_project()
                .expect("read state")
                .expect("project")
                .imported_pdf_path
                .as_deref(),
            Some(second_copy.as_path())
        );

        fs::remove_dir_all(root).expect("remove test directory");
    }
}
