//! Locked user-facing messages for typed service errors, so every screen shows
//! the same wording. UI tracks must call these instead of formatting errors.

use crate::{CommitError, GateReason, HistoryError, UndoError};

pub fn commit_error_message(error: &CommitError) -> String {
    match error {
        CommitError::WorkbookLocked(_) => {
            "The workbook is open in another program. Close it, then press Approve again. Your preview is kept.".into()
        }
        CommitError::ExternalModification => {
            "The workbook was changed outside Document Harvester since the last capture, so nothing was written. Restore the file or start a new project.".into()
        }
        CommitError::StalePreview => "This preview is out of date. Select the region again.".into(),
        CommitError::GateDisabled(reason) => reason.message.clone(),
        CommitError::ConfirmationRequired(_) => {
            "Please confirm the warnings before approving.".into()
        }
        CommitError::LimitExceeded(detail) => format!("An Excel limit was exceeded: {detail}"),
        CommitError::StateUnavailable(detail) => {
            format!("Project data could not be read: {detail}")
        }
        CommitError::Io(detail) => format!("File error: {detail}"),
    }
}

pub fn undo_error_message(error: &UndoError) -> String {
    match error {
        UndoError::NothingToUndo => "There is no capture to undo.".into(),
        UndoError::WorkbookLocked(_) => {
            "The workbook is open in another program. Close it, then press Undo again.".into()
        }
        UndoError::ExternalModification => {
            "The workbook was changed outside Document Harvester, so undo was refused and nothing was changed.".into()
        }
        UndoError::StateUnavailable(detail) => format!("Project data could not be read: {detail}"),
        UndoError::Io(detail) => format!("File error: {detail}"),
    }
}

pub fn history_error_message(error: &HistoryError) -> String {
    match error {
        HistoryError::StateUnavailable(detail) => {
            format!("Capture history could not be read: {detail}")
        }
    }
}

pub fn gate_reason_line(reason: &GateReason) -> String {
    format!("{}: {}", reason.code, reason.message)
}
