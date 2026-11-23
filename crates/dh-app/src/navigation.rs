//! Top-level screen navigation for the desktop shell (Stage 1, task 1.2).
//!
//! The shell has two destinations and one placeholder:
//!
//! - **Project home** — no project is open; the New Project entry point.
//! - **Project workspace** — a project is open; its identity, workbook and
//!   empty state are shown, and later stages add the region workflow here.
//! - **About placeholder** — the existing About panel, reachable from both
//!   screens.
//!
//! Screen choice, action availability and empty-state text live here, in the
//! application layer, so they can be tested without a running Slint window.
//! `dh-ui` only renders the resulting state and reports user intent.
//!
//! Nothing in this module reads a PDF or a workbook. PDF import is available in
//! the workspace from Stage 2; pointer-driven region selection remains later
//! work.

/// Which top-level screen the shell is showing.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Screen {
    /// No project is open: the New Project entry point.
    #[default]
    ProjectHome,
    /// A project is open; its identity and workbook are known.
    ProjectWorkspace,
}

/// Navigation-relevant events reported by the application layer.
///
/// The screen after each event is fixed by the event, so the resulting screen
/// does not depend on which copy of [`Navigation`] observed it.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NavigationEvent {
    /// Launch found no stored project.
    NoProjectStored,
    /// A project is open: just created, or restored with its workbook present.
    ProjectOpened,
    /// A stored project could not be opened: the private state is unreadable or
    /// the workbook it points at is missing.
    ProjectNotOpened,
    /// The user asked to start a new project.
    NewProjectRequested,
}

/// Current navigation state.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Navigation {
    screen: Screen,
}

impl Navigation {
    /// Start on project home, before any project is opened.
    pub fn new() -> Self {
        Self::default()
    }

    /// The screen currently shown.
    pub fn screen(self) -> Screen {
        self.screen
    }

    /// Whether a project is open, as the shell's `project-open` flag.
    pub fn project_open(self) -> bool {
        self.screen == Screen::ProjectWorkspace
    }

    /// Apply an event and return the resulting screen.
    pub fn apply(&mut self, event: NavigationEvent) -> Screen {
        self.screen = match event {
            NavigationEvent::ProjectOpened => Screen::ProjectWorkspace,
            NavigationEvent::NoProjectStored
            | NavigationEvent::ProjectNotOpened
            | NavigationEvent::NewProjectRequested => Screen::ProjectHome,
        };
        self.screen
    }
}

/// A control whose availability depends on the screen or the current form.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Action {
    /// Create the project and its blank workbook.
    CreateProject,
    /// Import a source PDF into private application data (Stage 2).
    ImportPdf,
}

/// Whether an action is usable, hidden, or visible but disabled.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Availability {
    /// The action works on this screen.
    Enabled,
    /// Visible but disabled; the shell shows this explanation next to it.
    Disabled(&'static str),
    /// Not part of this screen.
    Hidden,
}

/// Explanation shown beside the workspace's PDF import control.
pub const IMPORT_PDF_NOTE: &str =
    "Import a digital PDF; a private copy is used by the viewer. No OCR.";

/// The parts of the project form that decide whether creation can proceed.
///
/// `Default` is an untouched, idle form.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct ProjectForm {
    name_present: bool,
    parent_directory_present: bool,
    busy: bool,
}

impl ProjectForm {
    /// Read the form state from the current field values.
    pub fn for_values(name: &str, parent_directory: &str, busy: bool) -> Self {
        Self {
            name_present: !name.trim().is_empty(),
            parent_directory_present: !parent_directory.trim().is_empty(),
            busy,
        }
    }
}

/// Availability of `action` on `screen` given the current form contents.
pub fn availability(screen: Screen, action: Action, form: ProjectForm) -> Availability {
    match (screen, action) {
        (Screen::ProjectHome, Action::CreateProject) => {
            if form.busy {
                Availability::Disabled("A project is being created.")
            } else if !form.name_present {
                Availability::Disabled("Enter a project name.")
            } else if !form.parent_directory_present {
                Availability::Disabled("Choose where the project folder will be created.")
            } else {
                Availability::Enabled
            }
        }
        (Screen::ProjectWorkspace, Action::ImportPdf) => Availability::Enabled,
        (Screen::ProjectHome, Action::ImportPdf)
        | (Screen::ProjectWorkspace, Action::CreateProject) => Availability::Hidden,
    }
}

/// Empty-state text for the workspace of a project with no content.
///
/// No capture or append path exists yet (Stage 5 owns append), so an open
/// project cannot contain captures. Stage 5 must replace this with real
/// capture state instead of the constant.
pub const EMPTY_PROJECT_MESSAGE: &str = "No captures yet.";

/// Empty-state text for `screen`; empty when the screen has no empty state.
pub fn empty_state_message(screen: Screen) -> &'static str {
    match screen {
        Screen::ProjectHome => "",
        Screen::ProjectWorkspace => EMPTY_PROJECT_MESSAGE,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn form(name: &str, parent_directory: &str) -> ProjectForm {
        ProjectForm::for_values(name, parent_directory, false)
    }

    #[test]
    fn starts_on_project_home() {
        let navigation = Navigation::new();
        assert_eq!(navigation.screen(), Screen::ProjectHome);
        assert!(!navigation.project_open());
    }

    #[test]
    fn opening_a_project_shows_the_workspace() {
        let mut navigation = Navigation::new();
        assert_eq!(
            navigation.apply(NavigationEvent::ProjectOpened),
            Screen::ProjectWorkspace
        );
        assert!(navigation.project_open());
    }

    #[test]
    fn returning_from_the_workspace_shows_project_home() {
        let mut navigation = Navigation::new();
        navigation.apply(NavigationEvent::ProjectOpened);

        assert_eq!(
            navigation.apply(NavigationEvent::NewProjectRequested),
            Screen::ProjectHome
        );
        assert!(!navigation.project_open());
    }

    #[test]
    fn restoring_puts_the_shell_in_the_workspace() {
        let mut navigation = Navigation::new();
        assert_eq!(
            navigation.apply(NavigationEvent::ProjectOpened),
            Screen::ProjectWorkspace
        );
    }

    #[test]
    fn a_project_that_cannot_be_opened_stays_on_project_home() {
        let mut navigation = Navigation::new();
        assert_eq!(
            navigation.apply(NavigationEvent::ProjectNotOpened),
            Screen::ProjectHome
        );

        let mut navigation = Navigation::new();
        assert_eq!(
            navigation.apply(NavigationEvent::NoProjectStored),
            Screen::ProjectHome
        );
    }

    #[test]
    fn create_project_is_enabled_only_with_a_name_and_a_folder() {
        assert_eq!(
            availability(
                Screen::ProjectHome,
                Action::CreateProject,
                form("Report", "/tmp")
            ),
            Availability::Enabled
        );
        assert_eq!(
            availability(Screen::ProjectHome, Action::CreateProject, form("", "/tmp")),
            Availability::Disabled("Enter a project name.")
        );
        assert_eq!(
            availability(
                Screen::ProjectHome,
                Action::CreateProject,
                form("   ", "/tmp")
            ),
            Availability::Disabled("Enter a project name.")
        );
        assert_eq!(
            availability(
                Screen::ProjectHome,
                Action::CreateProject,
                form("Report", "")
            ),
            Availability::Disabled("Choose where the project folder will be created.")
        );
    }

    #[test]
    fn create_project_is_disabled_while_creating() {
        let busy = ProjectForm::for_values("Report", "/tmp", true);
        assert_eq!(
            availability(Screen::ProjectHome, Action::CreateProject, busy),
            Availability::Disabled("A project is being created.")
        );
    }

    #[test]
    fn create_project_is_not_offered_in_the_workspace() {
        assert_eq!(
            availability(
                Screen::ProjectWorkspace,
                Action::CreateProject,
                form("Report", "/tmp")
            ),
            Availability::Hidden
        );
    }

    #[test]
    fn pdf_import_is_enabled_in_the_workspace() {
        let availability = availability(
            Screen::ProjectWorkspace,
            Action::ImportPdf,
            form("Report", "/tmp"),
        );
        assert_eq!(availability, Availability::Enabled);
        assert!(!IMPORT_PDF_NOTE.is_empty());
    }

    #[test]
    fn pdf_import_is_not_offered_on_project_home() {
        assert_eq!(
            availability(Screen::ProjectHome, Action::ImportPdf, form("", "")),
            Availability::Hidden
        );
    }

    #[test]
    fn the_empty_project_state_belongs_to_the_workspace_only() {
        assert_eq!(empty_state_message(Screen::ProjectHome), "");
        assert_eq!(
            empty_state_message(Screen::ProjectWorkspace),
            EMPTY_PROJECT_MESSAGE
        );
        assert!(!EMPTY_PROJECT_MESSAGE.is_empty());
    }
}
