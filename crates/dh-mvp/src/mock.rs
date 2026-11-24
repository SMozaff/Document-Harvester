//! In-memory implementations of the locked services, for parallel development.
//!
//! UI, preview and QA tracks build against these before the real services
//! exist. They follow the same guarantees as the real services and are checked
//! by the same [`crate::conformance`] suite.

use std::collections::HashMap;
use std::sync::Mutex;
use std::time::{SystemTime, UNIX_EPOCH};

use dh_ir::Result;

use crate::{
    capture_row_count, ApprovalGate, CaptureListTransaction, CaptureService, CaptureSummary,
    CommitError, CommitRequest, ExtractionRequest, ExtractionService, HistoryError, PreviewModel,
    TransactionError, TransactionOutcome, UndoError,
};
use dh_writer::CaptureRecord;

/// Returns a canned preview for any request, keeping the request's selection.
pub struct FixedExtractionService {
    pub rows: u32,
    pub cols: u32,
}

impl ExtractionService for FixedExtractionService {
    fn extract_region(&self, request: &ExtractionRequest) -> Result<PreviewModel> {
        request.selection.validate()?;
        Ok(PreviewModel::build(
            request.pdf_sha256.clone(),
            request.selection,
            "fixture-v1".into(),
            vec![crate::fixtures::sample_block(self.rows, self.cols)],
            Vec::new(),
            Vec::new(),
            None,
        ))
    }
}

#[derive(Default)]
struct ProjectState {
    captures: Vec<(CaptureSummary, PreviewModel)>,
    locked: bool,
    externally_modified: bool,
}

/// Capture service that keeps everything in memory.
#[derive(Default)]
pub struct InMemoryCaptureService {
    projects: Mutex<HashMap<String, ProjectState>>,
}

impl InMemoryCaptureService {
    pub fn new_project(&self, id: &str) {
        self.projects.lock().unwrap().entry(id.into()).or_default();
    }
    pub fn set_locked(&self, id: &str, locked: bool) {
        self.projects
            .lock()
            .unwrap()
            .entry(id.into())
            .or_default()
            .locked = locked;
    }
    pub fn set_externally_modified(&self, id: &str, value: bool) {
        self.projects
            .lock()
            .unwrap()
            .entry(id.into())
            .or_default()
            .externally_modified = value;
    }
}

fn now_ms() -> u128 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_millis())
        .unwrap_or(0)
}

impl CaptureService for InMemoryCaptureService {
    fn commit(&self, request: &CommitRequest) -> std::result::Result<CaptureSummary, CommitError> {
        if !request.preview.is_consistent() {
            return Err(CommitError::StalePreview);
        }
        match &request.preview.gate {
            ApprovalGate::Disabled(reason) => {
                return Err(CommitError::GateDisabled(reason.clone()))
            }
            ApprovalGate::RequiresConfirmation(reasons) if !request.confirmed => {
                return Err(CommitError::ConfirmationRequired(reasons.clone()))
            }
            _ => {}
        }
        let mut projects = self.projects.lock().unwrap();
        let state = projects
            .get_mut(&request.project_id)
            .ok_or_else(|| CommitError::StateUnavailable("unknown project".into()))?;
        if state.locked {
            return Err(CommitError::WorkbookLocked("workbook is open".into()));
        }
        if state.externally_modified {
            return Err(CommitError::ExternalModification);
        }
        let sequence = state.captures.len() as u64 + 1;
        let row_start = state
            .captures
            .last()
            .map(|(s, _)| s.row_end + 1)
            .unwrap_or(1);
        let rows = capture_row_count(&request.preview.tables);
        let summary = CaptureSummary {
            id: format!("capture-{sequence}"),
            commit_sequence: sequence,
            source_pdf_sha256: request.preview.pdf_sha256.clone(),
            page: request.preview.selection.page,
            region: request.preview.selection.region,
            row_start,
            row_end: row_start + rows - 1,
            table_count: request.preview.tables.len() as u32,
            committed_at_unix_ms: now_ms(),
        };
        state
            .captures
            .push((summary.clone(), request.preview.clone()));
        Ok(summary)
    }

    fn history(&self, project_id: &str) -> std::result::Result<Vec<CaptureSummary>, HistoryError> {
        let projects = self.projects.lock().unwrap();
        let state = projects
            .get(project_id)
            .ok_or_else(|| HistoryError::StateUnavailable("unknown project".into()))?;
        Ok(state.captures.iter().map(|(s, _)| s.clone()).collect())
    }

    fn undo_latest(&self, project_id: &str) -> std::result::Result<CaptureSummary, UndoError> {
        let mut projects = self.projects.lock().unwrap();
        let state = projects
            .get_mut(project_id)
            .ok_or_else(|| UndoError::StateUnavailable("unknown project".into()))?;
        if state.captures.is_empty() {
            return Err(UndoError::NothingToUndo);
        }
        if state.locked {
            return Err(UndoError::WorkbookLocked("workbook is open".into()));
        }
        if state.externally_modified {
            return Err(UndoError::ExternalModification);
        }
        Ok(state.captures.pop().map(|(s, _)| s).unwrap())
    }
}

#[derive(Default)]
struct TxProject {
    list: Vec<CaptureRecord>,
    version: u64,
    locked: bool,
    externally_modified: bool,
    crash: Option<crate::conformance::CrashPoint>,
}

/// In-memory [`CaptureListTransaction`] with failure injection.
#[derive(Default)]
pub struct InMemoryTransaction {
    projects: Mutex<HashMap<String, TxProject>>,
}

impl InMemoryTransaction {
    pub fn new_project(&self, id: &str) {
        self.projects.lock().unwrap().entry(id.into()).or_default();
    }
    pub fn set_locked(&self, id: &str, locked: bool) {
        self.projects
            .lock()
            .unwrap()
            .entry(id.into())
            .or_default()
            .locked = locked;
    }
    pub fn set_externally_modified(&self, id: &str, value: bool) {
        self.projects
            .lock()
            .unwrap()
            .entry(id.into())
            .or_default()
            .externally_modified = value;
    }
    pub fn crash_next_apply(&self, id: &str, point: crate::conformance::CrashPoint) {
        self.projects
            .lock()
            .unwrap()
            .entry(id.into())
            .or_default()
            .crash = Some(point);
    }
}

impl CaptureListTransaction for InMemoryTransaction {
    fn load(&self, project_id: &str) -> std::result::Result<Vec<CaptureRecord>, TransactionError> {
        let projects = self.projects.lock().unwrap();
        projects
            .get(project_id)
            .map(|p| p.list.clone())
            .ok_or_else(|| TransactionError::StateUnavailable("unknown project".into()))
    }

    fn apply(
        &self,
        project_id: &str,
        new_list: &[CaptureRecord],
    ) -> std::result::Result<TransactionOutcome, TransactionError> {
        use crate::conformance::CrashPoint;
        let mut projects = self.projects.lock().unwrap();
        let project = projects
            .get_mut(project_id)
            .ok_or_else(|| TransactionError::StateUnavailable("unknown project".into()))?;
        if project.locked {
            return Err(TransactionError::WorkbookLocked("workbook is open".into()));
        }
        if project.externally_modified {
            return Err(TransactionError::ExternalModification);
        }
        match project.crash.take() {
            Some(CrashPoint::BeforeWorkbookReplace) => {
                return Err(TransactionError::Io("injected crash".into()));
            }
            Some(CrashPoint::AfterWorkbookReplace) => {
                project.list = new_list.to_vec();
                project.version += 1;
                return Err(TransactionError::Io("injected crash".into()));
            }
            None => {}
        }
        project.list = new_list.to_vec();
        project.version += 1;
        Ok(TransactionOutcome {
            workbook_sha256: format!("mem-{}", project.version),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::conformance::{run_conformance, ServiceHarness};

    struct Harness(InMemoryCaptureService);

    impl ServiceHarness for Harness {
        fn service(&self) -> &dyn CaptureService {
            &self.0
        }
        fn new_project(&self, name: &str) -> String {
            self.0.new_project(name);
            name.into()
        }
        fn set_locked(&self, project_id: &str, locked: bool) {
            self.0.set_locked(project_id, locked);
        }
        fn simulate_external_edit(&self, project_id: &str) {
            self.0.set_externally_modified(project_id, true);
        }
        fn clear_external_edit(&self, project_id: &str) {
            self.0.set_externally_modified(project_id, false);
        }
    }

    #[test]
    fn in_memory_service_passes_the_conformance_suite() {
        run_conformance(&Harness(InMemoryCaptureService::default()));
    }

    struct TxHarness(InMemoryTransaction);

    impl crate::conformance::TransactionHarness for TxHarness {
        fn transaction(&self) -> &dyn CaptureListTransaction {
            &self.0
        }
        fn new_project(&self, name: &str) -> String {
            self.0.new_project(name);
            name.into()
        }
        fn set_locked(&self, project_id: &str, locked: bool) {
            self.0.set_locked(project_id, locked);
        }
        fn simulate_external_edit(&self, project_id: &str) {
            self.0.set_externally_modified(project_id, true);
        }
        fn clear_external_edit(&self, project_id: &str) {
            self.0.set_externally_modified(project_id, false);
        }
        fn crash_next_apply(&self, project_id: &str, point: crate::conformance::CrashPoint) {
            self.0.crash_next_apply(project_id, point);
        }
    }

    #[test]
    fn in_memory_transaction_passes_the_transaction_conformance_suite() {
        crate::conformance::run_transaction_conformance(&TxHarness(InMemoryTransaction::default()));
    }

    #[test]
    fn fixed_extraction_returns_a_consistent_allowed_preview() {
        let preview = FixedExtractionService { rows: 3, cols: 2 }
            .extract_region(&crate::fixtures::sample_request(1))
            .unwrap();
        assert!(preview.is_consistent());
        assert_eq!(preview.gate, ApprovalGate::Allowed);
    }
}
