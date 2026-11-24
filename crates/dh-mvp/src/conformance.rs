//! Behavioral conformance suite for any [`CaptureService`].
//!
//! The real service (in `dh-project`) must pass this suite with a harness that
//! drives a real project directory, a real workbook and real lock/edit
//! simulation. Scenarios panic with a message naming the broken guarantee.

use crate::fixtures::sample_preview;
use crate::{
    ApprovalGate, CaptureListTransaction, CaptureService, CommitError, CommitRequest, GateReason,
    TransactionError, UndoError,
};
use dh_writer::CaptureRecord;

/// Test harness a real or mock service provides.
pub trait ServiceHarness {
    fn service(&self) -> &dyn CaptureService;
    /// Create a fresh project (blank workbook) and return its id.
    fn new_project(&self, name: &str) -> String;
    /// Make the workbook appear locked (open in Excel/LibreOffice) or not.
    fn set_locked(&self, project_id: &str, locked: bool);
    /// Change the workbook outside the app so its hash no longer matches.
    fn simulate_external_edit(&self, project_id: &str);
    /// Restore the workbook to its last committed bytes.
    fn clear_external_edit(&self, project_id: &str);
}

fn request(project: &str, rows: u32) -> CommitRequest {
    CommitRequest {
        project_id: project.into(),
        preview: sample_preview(1, rows, 2),
        confirmed: false,
    }
}

pub fn run_conformance<H: ServiceHarness>(h: &H) {
    let svc = h.service();

    // 1. New project: empty history, nothing to undo.
    let p = h.new_project("conf-1");
    assert!(
        svc.history(&p).unwrap().is_empty(),
        "new project history must be empty"
    );
    assert_eq!(
        svc.undo_latest(&p),
        Err(UndoError::NothingToUndo),
        "undo on empty"
    );

    // 2. Two commits: contiguous rows, ordered sequence, no separator row.
    let first = svc.commit(&request(&p, 3)).expect("first commit");
    let second = svc.commit(&request(&p, 2)).expect("second commit");
    assert_eq!((first.commit_sequence, second.commit_sequence), (1, 2));
    assert_eq!((first.row_start, first.row_end), (1, 3), "first rows");
    assert_eq!(
        second.row_start,
        first.row_end + 1,
        "no separator row between captures"
    );
    assert_eq!(
        second.row_end,
        second.row_start + 1,
        "second capture row count"
    );
    let history = svc.history(&p).unwrap();
    assert_eq!(
        history,
        vec![first.clone(), second.clone()],
        "history order"
    );

    // 3. Stale preview is refused and changes nothing.
    let mut stale = request(&p, 2);
    stale.preview.selection.region.x0 += 1.0;
    assert_eq!(
        svc.commit(&stale),
        Err(CommitError::StalePreview),
        "stale preview"
    );
    assert_eq!(
        svc.history(&p).unwrap().len(),
        2,
        "stale commit must not change history"
    );

    // 4. Gate handling.
    let mut disabled = request(&p, 2);
    disabled.preview.gate = ApprovalGate::Disabled(GateReason {
        code: "no-table-cells".into(),
        message: "m".into(),
    });
    assert!(
        matches!(svc.commit(&disabled), Err(CommitError::GateDisabled(_))),
        "disabled gate"
    );
    let mut confirm = request(&p, 2);
    confirm.preview.gate = ApprovalGate::RequiresConfirmation(vec![GateReason {
        code: "partial-cell".into(),
        message: "m".into(),
    }]);
    assert!(
        matches!(
            svc.commit(&confirm),
            Err(CommitError::ConfirmationRequired(_))
        ),
        "confirmation required"
    );
    confirm.confirmed = true;
    let third = svc.commit(&confirm).expect("confirmed commit");
    assert_eq!(third.commit_sequence, 3);

    // 5. Locked workbook: commit and undo refuse, nothing changes, retry works.
    h.set_locked(&p, true);
    assert!(
        matches!(
            svc.commit(&request(&p, 1)),
            Err(CommitError::WorkbookLocked(_))
        ),
        "locked commit"
    );
    assert!(
        matches!(svc.undo_latest(&p), Err(UndoError::WorkbookLocked(_))),
        "locked undo"
    );
    assert_eq!(
        svc.history(&p).unwrap().len(),
        3,
        "locked ops must not change history"
    );
    h.set_locked(&p, false);
    let fourth = svc.commit(&request(&p, 1)).expect("commit after unlock");
    assert_eq!(fourth.commit_sequence, 4);

    // 6. External edit: both operations refuse and change nothing.
    h.simulate_external_edit(&p);
    assert_eq!(
        svc.commit(&request(&p, 1)),
        Err(CommitError::ExternalModification),
        "ext commit"
    );
    assert_eq!(
        svc.undo_latest(&p),
        Err(UndoError::ExternalModification),
        "ext undo"
    );
    assert_eq!(
        svc.history(&p).unwrap().len(),
        4,
        "ext ops must not change history"
    );
    h.clear_external_edit(&p);

    // 7. Undo removes the latest only, newest first, until empty.
    let removed = svc.undo_latest(&p).expect("undo latest");
    assert_eq!(
        removed.commit_sequence, 4,
        "undo removes the latest capture"
    );
    assert_eq!(svc.history(&p).unwrap().len(), 3);
    for expected in [3u64, 2, 1] {
        assert_eq!(svc.undo_latest(&p).unwrap().commit_sequence, expected);
    }
    assert_eq!(svc.undo_latest(&p), Err(UndoError::NothingToUndo));

    // 8. After undoing everything, rows start at 1 again.
    let again = svc.commit(&request(&p, 2)).expect("commit after full undo");
    assert_eq!(
        (again.row_start, again.row_end),
        (1, 2),
        "rows restart after full undo"
    );
}

// ------------------------------------------------------------ transaction

/// Where an injected crash interrupts `apply`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CrashPoint {
    /// Temp workbook written, real workbook and state untouched.
    BeforeWorkbookReplace,
    /// Workbook replaced on disk, private state not yet updated.
    AfterWorkbookReplace,
}

pub trait TransactionHarness {
    fn transaction(&self) -> &dyn CaptureListTransaction;
    fn new_project(&self, name: &str) -> String;
    fn set_locked(&self, project_id: &str, locked: bool);
    fn simulate_external_edit(&self, project_id: &str);
    fn clear_external_edit(&self, project_id: &str);
    /// Make the next `apply` fail as if the process died at `point`, leaving
    /// the on-disk files exactly as a real crash there would.
    fn crash_next_apply(&self, project_id: &str, point: CrashPoint);
}

fn record(sequence: u64, rows: u32) -> CaptureRecord {
    let preview = sample_preview(1, rows, 2);
    CaptureRecord {
        id: format!("capture-{sequence}"),
        source_pdf_sha256: preview.pdf_sha256.clone(),
        page: preview.selection.page,
        region: preview.selection.region,
        commit_sequence: sequence,
        blocks: preview.tables,
    }
}

pub fn run_transaction_conformance<H: TransactionHarness>(h: &H) {
    let tx = h.transaction();
    let p = h.new_project("tx-1");

    // 1. Fresh project loads an empty list.
    assert!(
        tx.load(&p).expect("load fresh").is_empty(),
        "fresh list empty"
    );

    // 2. apply replaces the list and reports a workbook hash.
    let one = vec![record(1, 3)];
    let outcome = tx.apply(&p, &one).expect("apply one");
    assert!(!outcome.workbook_sha256.is_empty(), "hash reported");
    assert_eq!(tx.load(&p).unwrap(), one, "load returns applied list");
    let two = vec![record(1, 3), record(2, 2)];
    tx.apply(&p, &two).expect("apply two");
    assert_eq!(tx.load(&p).unwrap(), two);

    // 3. Lock: refuses, changes nothing, succeeds after unlock.
    h.set_locked(&p, true);
    assert!(
        matches!(tx.apply(&p, &one), Err(TransactionError::WorkbookLocked(_))),
        "lock"
    );
    assert_eq!(tx.load(&p).unwrap(), two, "lock must not change the list");
    h.set_locked(&p, false);
    tx.apply(&p, &one).expect("apply after unlock");
    assert_eq!(tx.load(&p).unwrap(), one);

    // 4. External edit: refuses and changes nothing.
    h.simulate_external_edit(&p);
    assert_eq!(
        tx.apply(&p, &two),
        Err(TransactionError::ExternalModification),
        "external edit"
    );
    assert_eq!(
        tx.load(&p).unwrap(),
        one,
        "external edit must not change the list"
    );
    h.clear_external_edit(&p);

    // 5. Crash before the workbook replace: old list survives, retry works.
    h.crash_next_apply(&p, CrashPoint::BeforeWorkbookReplace);
    assert!(
        tx.apply(&p, &two).is_err(),
        "injected crash must surface as an error"
    );
    assert_eq!(
        tx.load(&p).unwrap(),
        one,
        "crash before replace keeps the old list"
    );
    tx.apply(&p, &two)
        .expect("retry after crash before replace");
    assert_eq!(tx.load(&p).unwrap(), two);

    // 6. Crash after the workbook replace: recovery rolls forward to the new list.
    let three = vec![record(1, 3), record(2, 2), record(3, 1)];
    h.crash_next_apply(&p, CrashPoint::AfterWorkbookReplace);
    assert!(
        tx.apply(&p, &three).is_err(),
        "injected crash must surface as an error"
    );
    assert_eq!(
        tx.load(&p).unwrap(),
        three,
        "crash after replace rolls forward"
    );
    tx.apply(&p, &two).expect("apply after recovery");
    assert_eq!(tx.load(&p).unwrap(), two);
}
