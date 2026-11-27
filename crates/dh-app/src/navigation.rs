//! Top-level screen navigation for the desktop shell.
//!
//! The Rust application owns the active destination and project availability.
//! Slint renders that state and reports navigation intent without duplicating
//! the screen model.
//!
//! A project opened after a failed pending-transaction recovery stays on the
//! workspace screen in [`WorkspaceAccess::ReadOnly`]: every modifying action
//! (commit, undo, import) is disabled with one explanation. Navigation never
//! runs recovery itself; `dh-app` routes every open through
//! `dh_project::ProjectStore::open_project` before showing the workspace.
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
    /// Read-only capture preview and approval surface.
    Review,
    /// Dedicated capture history and guarded undo surface.
    History,
    /// Product information surface.
    About,
}

/// Navigation-relevant events reported by the application layer.
///
/// The screen after each event is fixed by the event, so the resulting screen
/// does not depend on which copy of [`Navigation`] observed it.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NavigationEvent {
    /// Launch found no stored project.
    NoProjectStored,
    /// A project is open: just created, or restored with its workbook present,
    /// and pending-transaction recovery succeeded.
    ProjectOpened,
    /// A project is open but pending-transaction recovery failed: show the
    /// workspace read-only with the reason from the open entry point.
    ProjectOpenedReadOnly,
    /// A stored project could not be opened: the private state is unreadable or
    /// the workbook it points at is missing.
    ProjectNotOpened,
    /// The user asked to start a new project.
    NewProjectRequested,
}

/// Whether the open project may be modified.
///
/// `ReadOnly` is entered only when opening a project had to continue after a
/// failed pending-transaction recovery. The project stays visible so the user
/// can see the workbook and the error, but commit, undo and import are disabled.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum WorkspaceAccess {
    #[default]
    Writable,
    ReadOnly,
}

/// Current navigation state. Project availability is independent of the
/// active destination so About and Project Home can be opened without closing
/// the current project.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Navigation {
    screen: Screen,
    access: WorkspaceAccess,
    project_open: bool,
    about_return_screen: Screen,
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

    /// Whether the open project may be modified.
    pub fn workspace_access(self) -> WorkspaceAccess {
        self.access
    }

    /// Whether a project is open, as the shell's `project-open` flag.
    pub fn project_open(self) -> bool {
        self.project_open
    }

    /// Apply an event and return the resulting screen.
    pub fn apply(&mut self, event: NavigationEvent) -> Screen {
        (
            self.screen,
            self.access,
            self.project_open,
            self.about_return_screen,
        ) = match event {
            NavigationEvent::ProjectOpened => (
                Screen::ProjectWorkspace,
                WorkspaceAccess::Writable,
                true,
                Screen::ProjectWorkspace,
            ),
            NavigationEvent::ProjectOpenedReadOnly => (
                Screen::ProjectWorkspace,
                WorkspaceAccess::ReadOnly,
                true,
                Screen::ProjectWorkspace,
            ),
            NavigationEvent::NoProjectStored
            | NavigationEvent::ProjectNotOpened
            | NavigationEvent::NewProjectRequested => (
                Screen::ProjectHome,
                WorkspaceAccess::Writable,
                false,
                Screen::ProjectHome,
            ),
        };
        self.screen
    }

    /// Navigate to a requested destination. Project-dependent screens safely
    /// fall back to Project Home when no project is open.
    pub fn navigate_to(&mut self, requested: Screen) -> Screen {
        match requested {
            Screen::About => {
                if self.screen != Screen::About {
                    self.about_return_screen = self.screen;
                }
                self.screen = Screen::About;
            }
            Screen::ProjectHome => self.screen = Screen::ProjectHome,
            Screen::ProjectWorkspace | Screen::Review | Screen::History if self.project_open => {
                self.screen = requested;
            }
            Screen::ProjectWorkspace | Screen::Review | Screen::History => {
                self.screen = Screen::ProjectHome;
            }
        }
        self.screen
    }

    /// Follow the shell's return path: About restores its previous route;
    /// Review and History return to the active project's Workspace.
    pub fn go_back(&mut self) -> Screen {
        self.screen = match self.screen {
            Screen::About => {
                if self.project_open
                    || matches!(
                        self.about_return_screen,
                        Screen::ProjectHome | Screen::About
                    )
                {
                    self.about_return_screen
                } else {
                    Screen::ProjectHome
                }
            }
            Screen::Review | Screen::History if self.project_open => Screen::ProjectWorkspace,
            Screen::Review | Screen::History => Screen::ProjectHome,
            screen => screen,
        };
        self.screen
    }
}

/// A control whose availability depends on the screen, the workspace access or
/// the current form.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Action {
    /// Create the project and its blank workbook.
    CreateProject,
    /// Import a source PDF into private application data (Stage 2).
    ImportPdf,
    /// Commit an approved capture to the workbook (Stage 5).
    ///
    /// The workspace control is not built yet; the variant lets the same
    /// read-only access rule gate it as soon as it is.
    #[allow(dead_code)]
    CommitCapture,
    /// Undo the latest committed capture (Stage 5).
    ///
    /// The workspace control is not built yet; the variant lets the same
    /// read-only access rule gate it as soon as it is.
    #[allow(dead_code)]
    UndoCapture,
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

/// One explanation for every modifying action when recovery failed at open.
pub const READ_ONLY_NOTE: &str =
    "Pending transaction recovery failed; this project is open read-only. Commit, undo and import are disabled until the journal is resolved.";

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

/// Availability of `action` on `screen`, given the workspace access and the
/// current form contents.
pub fn availability(
    screen: Screen,
    access: WorkspaceAccess,
    action: Action,
    form: ProjectForm,
) -> Availability {
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
        (Screen::ProjectWorkspace, Action::ImportPdf)
        | (Screen::ProjectWorkspace, Action::CommitCapture)
        | (Screen::ProjectWorkspace, Action::UndoCapture) => match access {
            WorkspaceAccess::Writable => Availability::Enabled,
            WorkspaceAccess::ReadOnly => Availability::Disabled(READ_ONLY_NOTE),
        },
        (Screen::ProjectHome, Action::ImportPdf)
        | (Screen::ProjectHome, Action::CommitCapture)
        | (Screen::ProjectHome, Action::UndoCapture)
        | (Screen::Review, Action::CreateProject)
        | (Screen::Review, Action::ImportPdf)
        | (Screen::Review, Action::CommitCapture)
        | (Screen::Review, Action::UndoCapture)
        | (Screen::History, Action::CreateProject)
        | (Screen::History, Action::ImportPdf)
        | (Screen::History, Action::CommitCapture)
        | (Screen::History, Action::UndoCapture)
        | (Screen::About, Action::CreateProject)
        | (Screen::About, Action::ImportPdf)
        | (Screen::About, Action::CommitCapture)
        | (Screen::About, Action::UndoCapture)
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
        Screen::ProjectHome | Screen::Review | Screen::History | Screen::About => "",
        Screen::ProjectWorkspace => EMPTY_PROJECT_MESSAGE,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn form(name: &str, parent_directory: &str) -> ProjectForm {
        ProjectForm::for_values(name, parent_directory, false)
    }

    fn writable(screen: Screen, action: Action, form: ProjectForm) -> Availability {
        availability(screen, WorkspaceAccess::Writable, action, form)
    }

    #[test]
    fn starts_on_project_home() {
        let navigation = Navigation::new();
        assert_eq!(navigation.screen(), Screen::ProjectHome);
        assert_eq!(navigation.workspace_access(), WorkspaceAccess::Writable);
        assert!(!navigation.project_open());
    }

    #[test]
    fn project_dependent_routes_fall_back_to_home_without_a_project() {
        let mut navigation = Navigation::new();
        for target in [Screen::ProjectWorkspace, Screen::Review, Screen::History] {
            assert_eq!(navigation.navigate_to(target), Screen::ProjectHome);
            assert!(!navigation.project_open());
        }
        assert_eq!(navigation.navigate_to(Screen::About), Screen::About);
        assert_eq!(navigation.go_back(), Screen::ProjectHome);
    }

    #[test]
    fn project_home_and_about_navigation_preserve_an_open_project() {
        let mut navigation = Navigation::new();
        navigation.apply(NavigationEvent::ProjectOpenedReadOnly);

        assert_eq!(
            navigation.navigate_to(Screen::ProjectHome),
            Screen::ProjectHome
        );
        assert!(navigation.project_open());
        assert_eq!(navigation.workspace_access(), WorkspaceAccess::ReadOnly);
        assert_eq!(navigation.navigate_to(Screen::About), Screen::About);
        assert_eq!(navigation.go_back(), Screen::ProjectHome);
        assert!(navigation.project_open());
    }

    #[test]
    fn about_returns_to_the_route_from_which_it_was_opened() {
        let mut navigation = Navigation::new();
        navigation.apply(NavigationEvent::ProjectOpened);

        assert_eq!(navigation.navigate_to(Screen::Review), Screen::Review);
        assert_eq!(navigation.navigate_to(Screen::About), Screen::About);
        assert_eq!(navigation.go_back(), Screen::Review);
        assert!(navigation.project_open());

        assert_eq!(navigation.navigate_to(Screen::About), Screen::About);
        assert_eq!(navigation.go_back(), Screen::Review);
    }

    #[test]
    fn history_and_review_return_to_the_workspace() {
        let mut navigation = Navigation::new();
        navigation.apply(NavigationEvent::ProjectOpened);

        assert_eq!(navigation.navigate_to(Screen::History), Screen::History);
        assert_eq!(navigation.go_back(), Screen::ProjectWorkspace);
        assert_eq!(navigation.navigate_to(Screen::Review), Screen::Review);
        assert_eq!(navigation.go_back(), Screen::ProjectWorkspace);
    }

    #[test]
    fn opening_a_different_project_resets_history_and_about_to_workspace() {
        let mut navigation = Navigation::new();
        navigation.apply(NavigationEvent::ProjectOpened);
        navigation.navigate_to(Screen::History);
        assert_eq!(
            navigation.apply(NavigationEvent::ProjectOpened),
            Screen::ProjectWorkspace
        );
        assert_eq!(navigation.screen(), Screen::ProjectWorkspace);
        assert_eq!(navigation.go_back(), Screen::ProjectWorkspace);

        navigation.navigate_to(Screen::About);
        assert_eq!(
            navigation.apply(NavigationEvent::ProjectOpened),
            Screen::ProjectWorkspace
        );
        assert_eq!(navigation.screen(), Screen::ProjectWorkspace);
        assert_eq!(navigation.go_back(), Screen::ProjectWorkspace);

        navigation.navigate_to(Screen::History);
        assert_eq!(
            navigation.apply(NavigationEvent::NewProjectRequested),
            Screen::ProjectHome
        );
        assert!(!navigation.project_open());
        navigation.apply(NavigationEvent::ProjectOpened);
        navigation.navigate_to(Screen::About);
        assert_eq!(
            navigation.apply(NavigationEvent::NewProjectRequested),
            Screen::ProjectHome
        );
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
        assert_eq!(navigation.workspace_access(), WorkspaceAccess::Writable);
    }

    #[test]
    fn a_read_only_open_still_shows_the_workspace() {
        let mut navigation = Navigation::new();
        assert_eq!(
            navigation.apply(NavigationEvent::ProjectOpenedReadOnly),
            Screen::ProjectWorkspace
        );
        assert!(navigation.project_open());
        assert_eq!(navigation.workspace_access(), WorkspaceAccess::ReadOnly);
    }

    #[test]
    fn returning_from_the_workspace_shows_project_home() {
        let mut navigation = Navigation::new();
        navigation.apply(NavigationEvent::ProjectOpenedReadOnly);

        assert_eq!(
            navigation.apply(NavigationEvent::NewProjectRequested),
            Screen::ProjectHome
        );
        assert!(!navigation.project_open());
        assert_eq!(navigation.workspace_access(), WorkspaceAccess::Writable);
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
            writable(
                Screen::ProjectHome,
                Action::CreateProject,
                form("Report", "/tmp")
            ),
            Availability::Enabled
        );
        assert_eq!(
            writable(Screen::ProjectHome, Action::CreateProject, form("", "/tmp")),
            Availability::Disabled("Enter a project name.")
        );
        assert_eq!(
            writable(
                Screen::ProjectHome,
                Action::CreateProject,
                form("   ", "/tmp")
            ),
            Availability::Disabled("Enter a project name.")
        );
        assert_eq!(
            writable(
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
            writable(Screen::ProjectHome, Action::CreateProject, busy),
            Availability::Disabled("A project is being created.")
        );
    }

    #[test]
    fn create_project_is_not_offered_in_the_workspace() {
        assert_eq!(
            writable(
                Screen::ProjectWorkspace,
                Action::CreateProject,
                form("Report", "/tmp")
            ),
            Availability::Hidden
        );
    }

    #[test]
    fn modifying_actions_are_enabled_in_a_writable_workspace() {
        for action in [
            Action::ImportPdf,
            Action::CommitCapture,
            Action::UndoCapture,
        ] {
            assert_eq!(
                writable(Screen::ProjectWorkspace, action, ProjectForm::default()),
                Availability::Enabled,
                "{action:?} must be available in a writable workspace"
            );
        }
        assert!(!IMPORT_PDF_NOTE.is_empty());
        assert!(!READ_ONLY_NOTE.is_empty());
    }

    #[test]
    fn read_only_workspace_disables_commit_undo_and_import() {
        for action in [
            Action::ImportPdf,
            Action::CommitCapture,
            Action::UndoCapture,
        ] {
            assert_eq!(
                availability(
                    Screen::ProjectWorkspace,
                    WorkspaceAccess::ReadOnly,
                    action,
                    ProjectForm::default(),
                ),
                Availability::Disabled(READ_ONLY_NOTE),
                "{action:?} must be refused after failed recovery"
            );
        }
    }

    #[test]
    fn modifying_actions_are_not_offered_on_project_home() {
        for action in [
            Action::ImportPdf,
            Action::CommitCapture,
            Action::UndoCapture,
        ] {
            assert_eq!(
                writable(Screen::ProjectHome, action, form("", "")),
                Availability::Hidden
            );
        }
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
