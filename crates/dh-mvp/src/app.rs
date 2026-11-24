//! Locked application workflow types and a reference workflow.
//!
//! The UI emits [`AppEvent`]s; a [`WorkflowCore`] turns each event into
//! [`Effect`]s; a runner executes service effects off the UI thread and feeds
//! results back as events; [`UiUpdate`]s tell the views what to show. Views and
//! services never call each other directly.

use std::path::PathBuf;

use crate::messages::{commit_error_message, history_error_message, undo_error_message};
use crate::{
    ApprovalGate, CaptureSummary, CommitError, CommitRequest, ExtractionRequest, HistoryError,
    PageSelection, PreviewModel, UndoError,
};

#[derive(Debug, Clone, PartialEq)]
#[allow(clippy::large_enum_variant)] // events are short-lived; boxing would complicate locked contract
pub enum AppEvent {
    /// A PDF is imported and open in the viewer.
    DocumentReady {
        project_id: String,
        pdf_path: PathBuf,
        pdf_sha256: String,
    },
    /// The viewer moved to another page; any selection and preview are void.
    PageChanged,
    /// The user finished drawing, moving or resizing the rectangle.
    SelectionChanged(PageSelection),
    SelectionCleared,
    ExtractionFinished(Result<PreviewModel, String>),
    ApproveRequested {
        confirmed: bool,
    },
    CancelRequested,
    RetryRequested,
    /// Back to the page to adjust the rectangle; keeps the selection.
    AdjustRequested,
    CommitFinished(Result<CaptureSummary, CommitError>),
    UndoRequested,
    UndoFinished(Result<CaptureSummary, UndoError>),
    HistoryRefreshRequested,
    HistoryLoaded(Result<Vec<CaptureSummary>, HistoryError>),
}

#[derive(Debug, Clone, PartialEq)]
#[allow(clippy::large_enum_variant)] // same rationale as AppEvent
pub enum Effect {
    RunExtraction(ExtractionRequest),
    RunCommit(CommitRequest),
    RunUndo { project_id: String },
    LoadHistory { project_id: String },
    Ui(UiUpdate),
}

#[derive(Debug, Clone, PartialEq)]
#[allow(clippy::large_enum_variant)] // same rationale as AppEvent
pub enum UiUpdate {
    /// Current rectangle in page space (`None` clears the overlay).
    Selection(Option<PageSelection>),
    Preview(PreviewModel),
    PreviewCleared,
    /// One-line message for the status area.
    Status(String),
    Busy(bool),
    History(Vec<CaptureSummary>),
}

pub trait WorkflowCore: Send {
    fn apply(&mut self, event: AppEvent) -> Vec<Effect>;
}

struct Document {
    project_id: String,
    pdf_path: PathBuf,
    pdf_sha256: String,
}

/// Minimal reference workflow so the application runs end to end with mocks.
/// The preview track replaces it with the full state machine.
#[derive(Default)]
pub struct BasicWorkflow {
    document: Option<Document>,
    selection: Option<PageSelection>,
    preview: Option<PreviewModel>,
}

impl BasicWorkflow {
    fn extraction(&self) -> Vec<Effect> {
        match (&self.document, self.selection) {
            (Some(doc), Some(selection)) => vec![
                Effect::Ui(UiUpdate::Busy(true)),
                Effect::RunExtraction(ExtractionRequest {
                    pdf_path: doc.pdf_path.clone(),
                    pdf_sha256: doc.pdf_sha256.clone(),
                    selection,
                }),
            ],
            _ => Vec::new(),
        }
    }

    fn reset(&mut self) -> Vec<Effect> {
        self.selection = None;
        self.preview = None;
        vec![
            Effect::Ui(UiUpdate::Selection(None)),
            Effect::Ui(UiUpdate::PreviewCleared),
            Effect::Ui(UiUpdate::Busy(false)),
        ]
    }
}

impl WorkflowCore for BasicWorkflow {
    fn apply(&mut self, event: AppEvent) -> Vec<Effect> {
        match event {
            AppEvent::DocumentReady {
                project_id,
                pdf_path,
                pdf_sha256,
            } => {
                let mut effects = self.reset();
                effects.push(Effect::LoadHistory {
                    project_id: project_id.clone(),
                });
                self.document = Some(Document {
                    project_id,
                    pdf_path,
                    pdf_sha256,
                });
                effects
            }
            AppEvent::PageChanged | AppEvent::SelectionCleared | AppEvent::CancelRequested => {
                self.reset()
            }
            AppEvent::SelectionChanged(selection) => {
                self.selection = Some(selection);
                self.preview = None;
                let mut effects = vec![
                    Effect::Ui(UiUpdate::Selection(Some(selection))),
                    Effect::Ui(UiUpdate::PreviewCleared),
                ];
                effects.extend(self.extraction());
                effects
            }
            AppEvent::RetryRequested => self.extraction(),
            AppEvent::AdjustRequested => {
                self.preview = None;
                vec![Effect::Ui(UiUpdate::PreviewCleared)]
            }
            AppEvent::ExtractionFinished(result) => match result {
                Ok(preview) if Some(preview.selection) == self.selection => {
                    self.preview = Some(preview.clone());
                    vec![
                        Effect::Ui(UiUpdate::Busy(false)),
                        Effect::Ui(UiUpdate::Preview(preview)),
                    ]
                }
                // A result for an older selection is stale and ignored.
                Ok(_) => Vec::new(),
                Err(message) => vec![
                    Effect::Ui(UiUpdate::Busy(false)),
                    Effect::Ui(UiUpdate::Status(message)),
                ],
            },
            AppEvent::ApproveRequested { confirmed } => match (&self.document, &self.preview) {
                (Some(doc), Some(preview))
                    if !matches!(preview.gate, ApprovalGate::Disabled(_)) =>
                {
                    vec![
                        Effect::Ui(UiUpdate::Busy(true)),
                        Effect::RunCommit(CommitRequest {
                            project_id: doc.project_id.clone(),
                            preview: preview.clone(),
                            confirmed,
                        }),
                    ]
                }
                _ => Vec::new(),
            },
            AppEvent::CommitFinished(Ok(summary)) => {
                let mut effects = self.reset();
                effects.push(Effect::Ui(UiUpdate::Status(format!(
                    "Captured rows {}-{}.",
                    summary.row_start, summary.row_end
                ))));
                if let Some(doc) = &self.document {
                    effects.push(Effect::LoadHistory {
                        project_id: doc.project_id.clone(),
                    });
                }
                effects
            }
            AppEvent::CommitFinished(Err(error)) => vec![
                Effect::Ui(UiUpdate::Busy(false)),
                Effect::Ui(UiUpdate::Status(commit_error_message(&error))),
            ],
            AppEvent::UndoRequested => match &self.document {
                Some(doc) => vec![
                    Effect::Ui(UiUpdate::Busy(true)),
                    Effect::RunUndo {
                        project_id: doc.project_id.clone(),
                    },
                ],
                None => Vec::new(),
            },
            AppEvent::UndoFinished(result) => {
                let mut effects = vec![Effect::Ui(UiUpdate::Busy(false))];
                match result {
                    Ok(removed) => {
                        effects.push(Effect::Ui(UiUpdate::Status(format!(
                            "Removed the capture at rows {}-{}.",
                            removed.row_start, removed.row_end
                        ))));
                        if let Some(doc) = &self.document {
                            effects.push(Effect::LoadHistory {
                                project_id: doc.project_id.clone(),
                            });
                        }
                    }
                    Err(error) => {
                        effects.push(Effect::Ui(UiUpdate::Status(undo_error_message(&error))))
                    }
                }
                effects
            }
            AppEvent::HistoryRefreshRequested => match &self.document {
                Some(doc) => vec![Effect::LoadHistory {
                    project_id: doc.project_id.clone(),
                }],
                None => Vec::new(),
            },
            AppEvent::HistoryLoaded(Ok(history)) => vec![Effect::Ui(UiUpdate::History(history))],
            AppEvent::HistoryLoaded(Err(error)) => {
                vec![Effect::Ui(UiUpdate::Status(history_error_message(&error)))]
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::fixtures::{sample_preview, sample_selection};

    fn ready() -> BasicWorkflow {
        let mut flow = BasicWorkflow::default();
        flow.apply(AppEvent::DocumentReady {
            project_id: "p".into(),
            pdf_path: "sample.pdf".into(),
            pdf_sha256: "sample-pdf-sha256".into(),
        });
        flow
    }

    #[test]
    fn selection_triggers_extraction_and_preview_is_shown() {
        let mut flow = ready();
        let effects = flow.apply(AppEvent::SelectionChanged(sample_selection(1)));
        assert!(effects
            .iter()
            .any(|e| matches!(e, Effect::RunExtraction(_))));
        let effects = flow.apply(AppEvent::ExtractionFinished(Ok(sample_preview(1, 2, 2))));
        assert!(effects
            .iter()
            .any(|e| matches!(e, Effect::Ui(UiUpdate::Preview(_)))));
    }

    #[test]
    fn stale_extraction_result_is_ignored() {
        let mut flow = ready();
        flow.apply(AppEvent::SelectionChanged(sample_selection(2)));
        let effects = flow.apply(AppEvent::ExtractionFinished(Ok(sample_preview(1, 2, 2))));
        assert!(effects.is_empty());
    }

    #[test]
    fn locked_workbook_keeps_the_preview_for_retry() {
        let mut flow = ready();
        flow.apply(AppEvent::SelectionChanged(sample_selection(1)));
        flow.apply(AppEvent::ExtractionFinished(Ok(sample_preview(1, 2, 2))));
        let effects = flow.apply(AppEvent::ApproveRequested { confirmed: false });
        assert!(effects.iter().any(|e| matches!(e, Effect::RunCommit(_))));
        flow.apply(AppEvent::CommitFinished(Err(CommitError::WorkbookLocked(
            "x".into(),
        ))));
        let effects = flow.apply(AppEvent::ApproveRequested { confirmed: false });
        assert!(
            effects.iter().any(|e| matches!(e, Effect::RunCommit(_))),
            "retry possible"
        );
    }

    #[test]
    fn successful_commit_clears_selection_and_reloads_history() {
        let mut flow = ready();
        flow.apply(AppEvent::SelectionChanged(sample_selection(1)));
        flow.apply(AppEvent::ExtractionFinished(Ok(sample_preview(1, 2, 2))));
        let summary = CaptureSummary {
            id: "capture-1".into(),
            commit_sequence: 1,
            source_pdf_sha256: "h".into(),
            page: 1,
            region: sample_selection(1).region,
            row_start: 1,
            row_end: 2,
            table_count: 1,
            committed_at_unix_ms: 0,
        };
        let effects = flow.apply(AppEvent::CommitFinished(Ok(summary)));
        assert!(effects
            .iter()
            .any(|e| matches!(e, Effect::LoadHistory { .. })));
        assert!(effects.contains(&Effect::Ui(UiUpdate::Selection(None))));
    }
}
