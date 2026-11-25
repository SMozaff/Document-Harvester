#![allow(dead_code, unused_variables)]

use std::{
    fs::{self, OpenOptions},
    path::{Path, PathBuf},
    time::{SystemTime, UNIX_EPOCH},
};

use calamine::{open_workbook, Data, Reader, Xlsx};
use dh_core::{config::JobConfig, DhError, Result};
use dh_ir::{BBox, CellState, TableBlock, UnassignedText};
use rust_xlsxwriter::{Workbook, Worksheet};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

const DESTINATION_SHEET_NAME: &str = "Sheet1";

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WorksheetLayout {
    pub sheet_name: String,
    pub page_header: String,
    pub spacer_rows: u32,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ReviewRow {
    pub sheet: String,
    pub page: u32,
    pub block: u32,
    pub row: u32,
    pub col: u32,
    pub cell_state: String,
    pub guard_status: String,
    pub reason: String,
    pub raw_text: String,
    pub raw_run_texts_json: String,
    pub written_text: String,
    pub numeric_source_text: String,
    pub numeric_canonical: String,
    pub fonts_json: String,
    pub extractor: String,
    pub extractor_version: String,
    pub policy_versions_json: String,
    pub raw_confidence_backend: String,
    pub raw_confidence_value: String,
    pub raw_confidence_semantics: String,
    pub calibrated_confidence: String,
}

/// One committed capture: the extracted table block(s) approved from one
/// source region and recorded in private project state.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CaptureRecord {
    pub id: String,
    pub source_pdf_sha256: String,
    pub page: u32,
    pub region: BBox,
    pub commit_sequence: u64,
    pub blocks: Vec<TableBlock>,
    /// Region-mode runs that could not be assigned to a reconstructed cell.
    /// They travel with the capture and journal payload so rebuild/recovery can
    /// reproduce the Review worksheet without a second state store.
    #[serde(default)]
    pub unassigned_text: Vec<UnassignedText>,
}

#[derive(Debug, Clone)]
pub struct WriteRequest<'a> {
    pub blocks: &'a [TableBlock],
    pub config: &'a JobConfig,
    pub output: &'a Path,
    pub include_review_sheet: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct XlsxWriter;

/// The value a rebuilt cell must carry: a real number or literal text.
#[derive(Debug, Clone, PartialEq)]
enum CellValue {
    Number(f64),
    String(String),
}

/// Outcome of a transaction-journal entry.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum JournalOutcome {
    Pending,
    Committed,
    RolledBack,
}

/// Version of the capture-list payload stored in a journal entry.
pub const JOURNAL_PAYLOAD_VERSION: u32 = 1;

/// The recovery source of truth for one journaled transaction: the full
/// ordered capture list that the workbook is being rebuilt from.
///
/// The payload is written with the pending journal entry before the workbook
/// write begins, so a crash after the workbook replacement can be recovered
/// from the journal alone.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct JournalPayload {
    pub version: u32,
    pub captures: Vec<CaptureRecord>,
}

impl JournalPayload {
    pub fn new(captures: Vec<CaptureRecord>) -> Self {
        Self {
            version: JOURNAL_PAYLOAD_VERSION,
            captures,
        }
    }
}

/// One transaction-journal entry for a commit or undo rebuild.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct JournalEntry {
    pub sequence: u64,
    pub commit_sequence: u64,
    pub outcome: JournalOutcome,
    pub workbook_sha256_after_commit: Option<String>,
    pub completed_unix_ms: Option<u64>,
    pub undone_commit_sequence: Option<u64>,
    /// Full capture list this transaction writes; absent only in journals
    /// written before the payload protocol existed.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub payload: Option<JournalPayload>,
}

/// Result of resolving the newest pending transaction-journal entry.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct JournalResolution {
    pub entry: JournalEntry,
    pub outcome: JournalOutcome,
}

/// Append-only JSON-lines transaction journal kept in private application data.
#[derive(Debug, Clone)]
pub struct TransactionJournal {
    path: PathBuf,
}

impl TransactionJournal {
    pub fn new(path: &Path) -> Self {
        Self {
            path: path.to_path_buf(),
        }
    }

    pub fn path(&self) -> &Path {
        &self.path
    }

    fn ensure_parent(&self) -> Result<()> {
        let parent = self
            .path
            .parent()
            .unwrap_or_else(|| std::path::Path::new("."));
        fs::create_dir_all(parent).map_err(|error| {
            DhError::State(format!(
                "cannot create transaction journal directory '{}': {error}",
                parent.display()
            ))
        })
    }

    pub fn load(&self) -> Result<Vec<JournalEntry>> {
        if !self.path.exists() {
            return Ok(Vec::new());
        }
        let bytes = fs::read(&self.path).map_err(|error| {
            DhError::State(format!(
                "cannot read transaction journal '{}': {error}",
                self.path.display()
            ))
        })?;
        let text = String::from_utf8_lossy(&bytes);
        let mut entries = Vec::new();
        for line in text.lines() {
            let line = line.trim();
            if line.is_empty() {
                continue;
            }
            let entry: JournalEntry = serde_json::from_str(line).map_err(|error| {
                DhError::State(format!(
                    "invalid transaction journal entry in '{}': {error}",
                    self.path.display()
                ))
            })?;
            entries.push(entry);
        }
        Ok(entries)
    }

    pub fn save(&self, entries: &[JournalEntry]) -> Result<()> {
        let mut content = String::new();
        for entry in entries {
            content.push_str(&serde_json::to_string(entry).map_err(|error| {
                DhError::State(format!(
                    "cannot serialize transaction journal entry: {error}"
                ))
            })?);
            content.push('\n');
        }
        self.ensure_parent()?;
        fs::write(&self.path, content).map_err(|error| {
            DhError::State(format!(
                "cannot write transaction journal '{}': {error}",
                self.path.display()
            ))
        })
    }

    pub fn next_sequence(&self, entries: &[JournalEntry]) -> Result<u64> {
        let last = entries
            .iter()
            .rev()
            .find_map(|entry| {
                if entry.sequence == u64::MAX {
                    None
                } else {
                    Some(entry.sequence)
                }
            })
            .unwrap_or(0);
        Ok(last + 1)
    }

    pub fn append(&self, entries: &mut Vec<JournalEntry>, entry: JournalEntry) -> Result<()> {
        entries.push(entry);
        self.save(entries)
    }

    pub fn mark_committed(
        &self,
        entries: &mut [JournalEntry],
        sequence: u64,
        workbook_sha256: &str,
        completed_unix_ms: u64,
    ) -> Result<()> {
        let Some(position) = entries.iter().position(|entry| entry.sequence == sequence) else {
            return Err(DhError::State(format!(
                "transaction journal entry {sequence} is not present"
            )));
        };
        entries[position].outcome = JournalOutcome::Committed;
        entries[position].workbook_sha256_after_commit = Some(workbook_sha256.to_string());
        entries[position].completed_unix_ms = Some(completed_unix_ms);
        self.save(entries)
    }

    pub fn mark_rolled_back(
        &self,
        entries: &mut [JournalEntry],
        sequence: u64,
        completed_unix_ms: u64,
    ) -> Result<()> {
        let Some(position) = entries.iter().position(|entry| entry.sequence == sequence) else {
            return Err(DhError::State(format!(
                "transaction journal entry {sequence} is not present"
            )));
        };
        entries[position].outcome = JournalOutcome::RolledBack;
        entries[position].workbook_sha256_after_commit = None;
        entries[position].completed_unix_ms = Some(completed_unix_ms);
        self.save(entries)
    }

    pub fn save_pending_with_sha(
        &self,
        entries: &mut [JournalEntry],
        sequence: u64,
        workbook_sha256: &str,
        completed_unix_ms: u64,
    ) -> Result<()> {
        let Some(position) = entries.iter().position(|entry| entry.sequence == sequence) else {
            return Err(DhError::State(format!(
                "transaction journal entry {sequence} is not present"
            )));
        };
        entries[position].outcome = JournalOutcome::Pending;
        entries[position].workbook_sha256_after_commit = Some(workbook_sha256.to_string());
        entries[position].completed_unix_ms = Some(completed_unix_ms);
        self.save(entries)
    }

    /// Resolve the newest pending journal entry against the on-disk workbook.
    ///
    /// A pending entry whose recorded post-commit hash matches the on-disk
    /// workbook is adopted as committed; any other pending entry is rolled
    /// back to the last valid workbook. Older pending entries, if any, are
    /// resolved in the same pass.
    pub fn resolve_pending(
        &self,
        output: &Path,
        _committed_workbook_sha256: &str,
    ) -> Result<Option<JournalResolution>> {
        let mut entries = self.load()?;
        self.resolve_pending_entries(&mut entries, output)
    }

    fn resolve_pending_entries(
        &self,
        entries: &mut Vec<JournalEntry>,
        output: &Path,
    ) -> Result<Option<JournalResolution>> {
        loop {
            let Some(position) = entries
                .iter()
                .rposition(|entry| entry.outcome == JournalOutcome::Pending)
            else {
                return Ok(None);
            };
            let mut entry = entries.remove(position);
            let on_disk = try_sha256_file(output);
            let adopted = entry
                .workbook_sha256_after_commit
                .as_deref()
                .map(|sha| on_disk.as_deref() == Some(sha))
                .unwrap_or(false);
            let outcome = if adopted {
                JournalOutcome::Committed
            } else {
                JournalOutcome::RolledBack
            };
            if adopted {
                entry.outcome = JournalOutcome::Committed;
                entry.completed_unix_ms = Some(current_unix_ms());
                entry.workbook_sha256_after_commit = on_disk.clone();
            } else {
                entry.outcome = JournalOutcome::RolledBack;
                entry.completed_unix_ms = Some(current_unix_ms());
            }
            let resolution = JournalResolution {
                entry: entry.clone(),
                outcome,
            };
            entries.insert(position, entry);
            self.save(entries)?;
            if adopted {
                return Ok(Some(resolution));
            }
        }
    }

    /// Resolve a crashed transaction and return the entry whose payload
    /// describes the current on-disk workbook.
    ///
    /// Pending entries are resolved first (adopted when their recorded hash
    /// matches the workbook, otherwise rolled back). If no pending entry is
    /// adopted, the newest committed entry whose recorded hash matches the
    /// on-disk workbook and that carries a payload is returned, so private
    /// state can be restored after a crash between replacement and the state
    /// write. Returns `None` when no journaled payload describes the workbook
    /// (for example, before the first commit or for legacy hash-only entries).
    pub fn recover(&self, output: &Path) -> Result<Option<JournalResolution>> {
        let mut entries = self.load()?;
        if let Some(resolution) = self.resolve_pending_entries(&mut entries, output)? {
            return Ok(Some(resolution));
        }
        let on_disk = try_sha256_file(output);
        let Some(on_disk) = on_disk else {
            return Ok(None);
        };
        for entry in entries.iter().rev() {
            if entry.workbook_sha256_after_commit.as_deref() == Some(on_disk.as_str())
                && entry.payload.is_some()
            {
                return Ok(Some(JournalResolution {
                    outcome: entry.outcome,
                    entry: entry.clone(),
                }));
            }
        }
        Ok(None)
    }
}

/// Lock state of the destination workbook.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LockStatus {
    Unlocked,
    Locked,
}

/// Injected transaction failure points for tests.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FailureStage {
    BeforeTempWrite,
    AfterTempWrite,
    AfterVerification,
    AfterReplacement,
    BeforeJournalCompletion,
}

impl XlsxWriter {
    /// Create the one blank workbook owned by a new project.
    ///
    /// This is intentionally separate from append/region writing: the project
    /// lifecycle must establish a valid workbook before captures exist. An
    /// empty capture list produces a valid, empty workbook.
    pub fn create_blank(&self, output: &Path) -> Result<()> {
        let captures: Vec<CaptureRecord> = Vec::new();
        let _committed_hash =
            self.write_capture_list(&captures, output, &scaffold_job_config(), "")?;
        Ok(())
    }

    /// Rebuild the full workbook from the ordered capture list and commit it.
    ///
    /// The workbook is written to a temporary file, reopened with the
    /// independent reader (`calamine`) and verified cell by cell against the
    /// capture data, and only then atomically replaces the target. Any failure
    /// before the replace keeps the last valid workbook untouched. The
    /// external-change check refuses to overwrite a workbook whose content no
    /// longer matches the hash of the last committed workbook.
    ///
    /// Returns the SHA-256 hex digest of the committed workbook.
    pub fn write_capture_list(
        &self,
        captures: &[CaptureRecord],
        output: &Path,
        _config: &JobConfig,
        committed_workbook_sha256: &str,
    ) -> Result<String> {
        self.rebuild_and_commit(
            captures,
            output,
            committed_workbook_sha256,
            None,
            None,
            None,
            captures
                .last()
                .map(|capture| capture.commit_sequence)
                .unwrap_or(0),
            None,
        )
    }

    /// Rebuild the full workbook from the ordered capture list and commit it
    /// through the transaction journal.
    ///
    /// Before the temp write begins a pending journal entry is appended with
    /// the full capture-list payload; after the verified replacement it is
    /// marked committed (or, when a failure is injected before completion,
    /// left pending so the next startup resolves it against the on-disk
    /// workbook). A pending entry whose recorded post-commit hash matches the
    /// on-disk workbook is adopted as committed on restart; any other pending
    /// entry is rolled back to the last valid workbook. The payload is the
    /// recovery source of truth until private state records the commit.
    pub fn write_capture_list_with_journal(
        &self,
        captures: &[CaptureRecord],
        output: &Path,
        committed_workbook_sha256: &str,
        journal: &TransactionJournal,
        failure_stage: Option<FailureStage>,
        lock: Option<&dyn Fn(&Path) -> LockStatus>,
    ) -> Result<String> {
        self.rebuild_and_commit(
            captures,
            output,
            committed_workbook_sha256,
            Some(journal),
            failure_stage,
            lock,
            captures
                .last()
                .map(|capture| capture.commit_sequence)
                .unwrap_or(0),
            None,
        )
    }

    /// Undo the latest committed capture by rebuilding the workbook from the
    /// remaining captures through the same writer and verification path.
    ///
    /// `captures` is the full committed capture list; the latest entry is
    /// removed and the workbook is rebuilt from the remainder. The rebuild
    /// refuses an externally modified workbook and keeps pending-journal state
    /// consistent.
    pub fn undo_capture_list(
        &self,
        captures: &[CaptureRecord],
        output: &Path,
        committed_workbook_sha256: &str,
        journal: &TransactionJournal,
        failure_stage: Option<FailureStage>,
        lock: Option<&dyn Fn(&Path) -> LockStatus>,
    ) -> Result<String> {
        if captures.is_empty() {
            return Err(DhError::State("no committed captures to undo".to_string()));
        }
        let removed = captures[captures.len() - 1].clone();
        let remaining = captures[..captures.len() - 1].to_vec();
        self.rebuild_and_commit(
            &remaining,
            output,
            committed_workbook_sha256,
            Some(journal),
            failure_stage,
            lock,
            removed.commit_sequence,
            Some(removed.commit_sequence),
        )
    }

    /// Rebuild the workbook through the journaled transaction.
    ///
    /// Steps: lock check, startup pending-journal resolution, external-change
    /// check, append pending journal entry, temporary write, independent
    /// verification, atomic replacement, journal completion.
    #[allow(clippy::too_many_arguments)]
    fn rebuild_and_commit(
        &self,
        captures: &[CaptureRecord],
        output: &Path,
        committed_workbook_sha256: &str,
        journal: Option<&TransactionJournal>,
        failure_stage: Option<FailureStage>,
        lock: Option<&dyn Fn(&Path) -> LockStatus>,
        affected_commit_sequence: u64,
        undone_commit_sequence: Option<u64>,
    ) -> Result<String> {
        let lock_status = lock
            .map(|probe| probe(output))
            .unwrap_or_else(|| XlsxWriter::probe_workbook_lock(output));
        if lock_status != LockStatus::Unlocked {
            return Err(DhError::Write(format!(
                "workbook '{}' is locked; commit or undo deferred until the workbook is closed",
                output.display()
            )));
        }

        let mut entries: Vec<JournalEntry> = Vec::new();
        let mut effective_committed_hash = committed_workbook_sha256.trim().to_string();
        if let Some(journal) = journal {
            if let Some(resolution) = journal.resolve_pending(output, &effective_committed_hash)? {
                if resolution.outcome == JournalOutcome::Committed {
                    effective_committed_hash = resolution
                        .entry
                        .workbook_sha256_after_commit
                        .clone()
                        .unwrap_or_default();
                }
            }
            entries = journal.load()?;
            if entries
                .iter()
                .any(|entry| entry.outcome == JournalOutcome::Pending)
            {
                return Err(DhError::State(
                    "transaction journal has unresolved pending entries; refusing to mutate"
                        .to_string(),
                ));
            }
        }

        if !effective_committed_hash.is_empty() && output.exists() {
            if let Some(actual) = try_sha256_file(output) {
                if actual != effective_committed_hash {
                    return Err(DhError::State(format!(
                        "workbook '{}' does not match the hash of the last committed workbook; refusing to rebuild over an externally modified file",
                        output.display()
                    )));
                }
            }
        }

        let sequence: u64 = match journal {
            Some(journal) => journal.next_sequence(&entries)?,
            None => 0,
        };
        let entry = JournalEntry {
            sequence,
            commit_sequence: affected_commit_sequence,
            outcome: JournalOutcome::Pending,
            workbook_sha256_after_commit: None,
            completed_unix_ms: None,
            undone_commit_sequence,
            payload: Some(JournalPayload::new(captures.to_vec())),
        };
        if let Some(journal) = journal {
            journal.append(&mut entries, entry)?;
        }

        let temporary_path = temporary_workbook_path(output);
        if failure_stage == Some(FailureStage::BeforeTempWrite) {
            if let Some(journal) = journal {
                journal.mark_rolled_back(&mut entries, sequence, current_unix_ms())?;
            }
            return Err(DhError::Write(
                "injected failure before temporary workbook write".to_string(),
            ));
        }
        let write_result = self.rebuild_workbook(captures, &temporary_path, &scaffold_job_config());
        if let Err(error) = write_result {
            if let Some(journal) = journal {
                journal.mark_rolled_back(&mut entries, sequence, current_unix_ms())?;
            }
            let _ = fs::remove_file(&temporary_path);
            return Err(DhError::Write(format!(
                "cannot write rebuilt workbook '{}': {error}",
                temporary_path.display()
            )));
        }
        if failure_stage == Some(FailureStage::AfterTempWrite) {
            if let Some(journal) = journal {
                journal.mark_rolled_back(&mut entries, sequence, current_unix_ms())?;
            }
            let _ = fs::remove_file(&temporary_path);
            return Err(DhError::Write(
                "injected failure after temporary workbook write".to_string(),
            ));
        }
        let verify_result = self.verify_rebuilt(&temporary_path, captures);
        if let Err(error) = verify_result {
            if let Some(journal) = journal {
                journal.mark_rolled_back(&mut entries, sequence, current_unix_ms())?;
            }
            let _ = fs::remove_file(&temporary_path);
            return Err(error);
        }
        if failure_stage == Some(FailureStage::AfterVerification) {
            if let Some(journal) = journal {
                journal.mark_rolled_back(&mut entries, sequence, current_unix_ms())?;
            }
            let _ = fs::remove_file(&temporary_path);
            return Err(DhError::Write(
                "injected failure after verification".to_string(),
            ));
        }
        // Record the verified post-commit hash before the rename so that a
        // crash between the rename and journal completion can be resolved on
        // restart: a matching on-disk workbook is adopted, an older workbook
        // is rolled back.
        let new_hash = sha256_file(&temporary_path)?;
        if let Some(journal) = journal {
            journal.save_pending_with_sha(&mut entries, sequence, &new_hash, current_unix_ms())?;
        }
        let rename_result = fs::rename(&temporary_path, output);
        if let Err(error) = rename_result {
            if let Some(journal) = journal {
                journal.mark_rolled_back(&mut entries, sequence, current_unix_ms())?;
            }
            let _ = fs::remove_file(&temporary_path);
            return Err(DhError::Write(format!(
                "cannot install rebuilt workbook '{}': {error}",
                output.display()
            )));
        }
        if failure_stage == Some(FailureStage::AfterReplacement) {
            if let Some(journal) = journal {
                journal.mark_committed(&mut entries, sequence, &new_hash, current_unix_ms())?;
            }
            return Err(DhError::Write(
                "injected failure after replacement before journal completion".to_string(),
            ));
        }
        if failure_stage == Some(FailureStage::BeforeJournalCompletion) {
            return Err(DhError::Write(
                "injected failure before journal completion".to_string(),
            ));
        }
        if let Some(journal) = journal {
            journal.mark_committed(&mut entries, sequence, &new_hash, current_unix_ms())?;
        }
        Ok(new_hash)
    }

    fn review_rows(&self, blocks: &[TableBlock]) -> Result<Vec<ReviewRow>> {
        todo!("XlsxWriter::review_rows")
    }

    fn apply_font_policy(&self, block: &TableBlock) -> Result<()> {
        todo!("XlsxWriter::apply_font_policy")
    }

    fn rebuild_workbook(
        &self,
        captures: &[CaptureRecord],
        path: &Path,
        _config: &JobConfig,
    ) -> Result<()> {
        let mut workbook = Workbook::new();
        {
            let sheet = workbook.add_worksheet();
            sheet.set_name(DESTINATION_SHEET_NAME).map_err(|error| {
                DhError::Write(format!("cannot set destination worksheet name: {error}"))
            })?;
            let plan = plan_cells(captures);
            for (row, col, value) in plan {
                match value {
                    CellValue::Number(number) => {
                        sheet.write_number(row, col, number).map_err(|error| {
                            DhError::Write(format!(
                                "cannot write rebuilt cell {row},{col}: {error}"
                            ))
                        })?;
                    }
                    CellValue::String(text) => {
                        write_text_cell(sheet, row, col, &text)?;
                    }
                }
            }
        }

        let unassigned: Vec<(&CaptureRecord, &UnassignedText)> = captures
            .iter()
            .flat_map(|capture| {
                capture
                    .unassigned_text
                    .iter()
                    .map(move |item| (capture, item))
            })
            .collect();
        if !unassigned.is_empty() {
            let sheet = workbook.add_worksheet();
            sheet.set_name("Review").map_err(|error| {
                DhError::Write(format!("cannot name Review worksheet: {error}"))
            })?;
            let headers = ["Page", "X0", "Y0", "X1", "Y1", "Reason", "Text", "Run ID"];
            for (column, header) in headers.iter().enumerate() {
                write_text_cell(sheet, 0, column as u16, header)?;
            }
            for (offset, (capture, item)) in unassigned.iter().enumerate() {
                let row = u32::try_from(offset + 1)
                    .map_err(|_| DhError::Write("too many Review rows".to_owned()))?;
                let values = [
                    (0u16, f64::from(capture.page)),
                    (1u16, item.bbox.x0),
                    (2u16, item.bbox.y0),
                    (3u16, item.bbox.x1),
                    (4u16, item.bbox.y1),
                ];
                for (column, value) in values {
                    sheet.write_number(row, column, value).map_err(|error| {
                        DhError::Write(format!(
                            "cannot write Review coordinate at row {row}: {error}"
                        ))
                    })?;
                }
                for (column, value) in [
                    (5u16, item.reason.as_str()),
                    (6u16, item.text.as_str()),
                    (7u16, item.run_id.as_str()),
                ] {
                    write_text_cell(sheet, row, column, value)?;
                }
            }
        }

        workbook.save(path).map_err(|error| {
            DhError::Write(format!(
                "cannot write rebuilt workbook '{}': {error}",
                path.display()
            ))
        })?;
        Ok(())
    }

    fn verify_rebuilt(&self, path: &Path, captures: &[CaptureRecord]) -> Result<()> {
        let mut workbook: Xlsx<_> = open_workbook(path).map_err(|error| {
            DhError::Write(format!(
                "verification failed: independent reader cannot reopen rebuilt workbook '{}': {error}",
                path.display()
            ))
        })?;
        let range = workbook
            .worksheet_range(DESTINATION_SHEET_NAME)
            .map_err(|error| {
                DhError::Write(format!(
                    "verification failed: cannot read rebuilt worksheet: {error}"
                ))
            })?;
        for (row, col, value) in plan_cells(captures) {
            let actual = range.get_value((row, u32::from(col)));
            let matches = match (&value, actual) {
                (CellValue::Number(expected), Some(Data::Int(actual))) => {
                    (*expected - (*actual as f64)).abs() < 1e-6
                }
                (CellValue::Number(expected), Some(Data::Float(actual))) => {
                    (*expected - actual).abs() < 1e-6
                }
                (CellValue::Number(_), _) => false,
                (CellValue::String(expected), Some(Data::String(actual))) => expected == actual,
                (CellValue::String(_), _) => false,
            };
            if !matches {
                return Err(DhError::Write(format!(
                    "verification failed: rebuilt cell {row},{col} does not match the capture data"
                )));
            }
        }

        // Independently verify that every Review record survived the XLSX
        // write, including its page, coordinates, reason and original text.
        let expected_review: Vec<(&CaptureRecord, &UnassignedText)> = captures
            .iter()
            .flat_map(|capture| {
                capture
                    .unassigned_text
                    .iter()
                    .map(move |item| (capture, item))
            })
            .collect();
        if !expected_review.is_empty() {
            let review = workbook.worksheet_range("Review").map_err(|error| {
                DhError::Write(format!(
                    "verification failed: Review worksheet is missing: {error}"
                ))
            })?;
            for (offset, (capture, item)) in expected_review.iter().enumerate() {
                let row = u32::try_from(offset + 1)
                    .map_err(|_| DhError::Write("too many Review rows".to_owned()))?;
                if !number_cell_matches(review.get_value((row, 0)), f64::from(capture.page))
                    || !number_cell_matches(review.get_value((row, 1)), item.bbox.x0)
                    || !number_cell_matches(review.get_value((row, 2)), item.bbox.y0)
                    || !number_cell_matches(review.get_value((row, 3)), item.bbox.x1)
                    || !number_cell_matches(review.get_value((row, 4)), item.bbox.y1)
                    || review.get_value((row, 5)) != Some(&Data::String(item.reason.clone()))
                    || review.get_value((row, 6)) != Some(&Data::String(item.text.clone()))
                    || review.get_value((row, 7)) != Some(&Data::String(item.run_id.clone()))
                {
                    return Err(DhError::Write(format!(
                        "verification failed: Review row {row} does not match the capture data"
                    )));
                }
            }
        }
        Ok(())
    }

    fn probe_workbook_lock(path: &Path) -> LockStatus {
        if !path.exists() {
            return LockStatus::Unlocked;
        }
        match OpenOptions::new().write(true).open(path) {
            Ok(handle) => {
                drop(handle);
                LockStatus::Unlocked
            }
            Err(_) => LockStatus::Locked,
        }
    }
}

fn scaffold_job_config() -> JobConfig {
    JobConfig {
        column_order: dh_core::config::ColumnOrder::LeftToRight,
        stitch_continued_tables: dh_core::config::StitchMode::Separate,
        guard_policy_flags: dh_core::config::GuardPolicyFlags {
            normalize_presentation_forms: false,
            normalize_arabic_variants: false,
            trim_output: false,
        },
        policy_versions: dh_core::config::PolicyVersions {
            tool_version: "scaffold".to_string(),
            crate_versions: Vec::new(),
            normalization_table_version: "scaffold".to_string(),
            validated_font_list_version: "scaffold".to_string(),
            run_join_policy_version: "scaffold".to_string(),
            cell_serialization_policy_version: "scaffold".to_string(),
            numeric_policy_version: "scaffold".to_string(),
            continuation_classifier_version: "scaffold".to_string(),
            continuation_threshold: "scaffold".to_string(),
            continuation_ambiguity_margin: "scaffold".to_string(),
            excel_overflow_policy: "scaffold".to_string(),
        },
    }
}

/// The workbook row range [start, end) allocated to one committed capture.
///
/// Captures are appended below the previous capture with no separator row, so
/// the start of a capture is the end of the previous capture.
pub fn capture_row_spans(captures: &[CaptureRecord]) -> Vec<(u32, u32)> {
    let mut spans = Vec::new();
    let mut cursor: u32 = 0;
    for capture in captures {
        let start = cursor;
        let mut end = cursor;
        for block in &capture.blocks {
            let block_start = cursor;
            let mut block_end_offset: u32 = 0;
            for row_cells in &block.rows {
                for cell in row_cells {
                    if let Some((grid_row, _grid_col)) = cell.grid_position {
                        block_end_offset = block_end_offset.max(grid_row + cell.rowspan);
                    }
                }
            }
            end = end.max(block_start + block_end_offset);
            cursor = block_start + block_end_offset;
        }
        spans.push((start, end));
    }
    spans
}

fn captures_from_blocks(blocks: &[TableBlock]) -> Vec<CaptureRecord> {
    blocks
        .iter()
        .enumerate()
        .map(|(index, block)| CaptureRecord {
            id: format!("capture-{}", index),
            source_pdf_sha256: String::new(),
            page: block.page,
            region: block.bbox,
            commit_sequence: index as u64 + 1,
            blocks: vec![block.clone()],
            unassigned_text: Vec::new(),
        })
        .collect()
}

fn temporary_workbook_path(output: &Path) -> PathBuf {
    let directory = output.parent().unwrap_or_else(|| Path::new("."));
    let file_name = output
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or("workbook.xlsx");
    directory.join(format!(".{file_name}.tmp-{}", std::process::id()))
}

/// SHA-256 of a file used to bind preview identity to its source/workbook.
pub fn file_sha256(path: &Path) -> Result<String> {
    sha256_file(path)
}

fn sha256_file(path: &Path) -> Result<String> {
    let bytes = std::fs::read(path).map_err(|error| {
        DhError::State(format!(
            "cannot read workbook '{}' for hashing: {error}",
            path.display()
        ))
    })?;
    let digest = Sha256::digest(&bytes);
    let hex = digest
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect::<String>();
    Ok(hex)
}

fn try_sha256_file(path: &Path) -> Option<String> {
    let bytes = std::fs::read(path).ok()?;
    let digest = Sha256::digest(&bytes);
    let hex = digest
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect::<String>();
    Some(hex)
}

fn current_unix_ms() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_millis() as u64)
        .unwrap_or(0)
}

const MAX_TEXT_UTF16_UNITS: usize = 32_767;

/// Write one literal text cell. All worksheet text writes pass through this
/// function so formula-like prefixes remain strings and Excel's UTF-16 cell
/// limit is enforced before the workbook library is called.
fn write_text_cell(sheet: &mut Worksheet, row: u32, column: u16, text: &str) -> Result<()> {
    let utf16_units = text.encode_utf16().count();
    if utf16_units > MAX_TEXT_UTF16_UNITS {
        return Err(DhError::Write(format!(
            "text cell {row},{column} contains {utf16_units} UTF-16 code units; Excel permits at most {MAX_TEXT_UTF16_UNITS}"
        )));
    }

    sheet
        .write_string(row, column, text)
        .map(|_| ())
        .map_err(|error| DhError::Write(format!("cannot write text cell {row},{column}: {error}")))
}

fn plan_cells(captures: &[CaptureRecord]) -> Vec<(u32, u16, CellValue)> {
    let mut plan = Vec::new();
    let mut cursor_row: u32 = 0;
    for capture in captures {
        for block in &capture.blocks {
            let block_start_row = cursor_row;
            let mut end_row: u32 = 0;
            for row_cells in block.rows.iter() {
                for cell in row_cells {
                    let Some((grid_row, grid_col)) = cell.grid_position else {
                        continue;
                    };
                    // A partial cell reserves its grid row but is always blank.
                    // Other cell row-span behavior remains the Stage 5 behavior.
                    if cell.state == CellState::Partial {
                        end_row = end_row.max(grid_row + cell.rowspan.saturating_sub(1));
                        continue;
                    }
                    let value = if let Some(info) = &cell.numeric {
                        CellValue::Number(info.value)
                    } else if let Some(text) = &cell.text {
                        CellValue::String(text.clone())
                    } else if let Some(raw) = &cell.raw {
                        CellValue::String(raw.clone())
                    } else {
                        continue;
                    };
                    plan.push((
                        block_start_row + grid_row,
                        grid_col.min(16_383) as u16,
                        value,
                    ));
                    end_row = end_row.max(grid_row + cell.rowspan - 1);
                }
            }
            cursor_row = block_start_row + end_row + 1;
        }
    }
    plan
}

fn number_cell_matches(actual: Option<&Data>, expected: f64) -> bool {
    match actual {
        Some(Data::Int(value)) => (expected - (*value as f64)).abs() < 1e-6,
        Some(Data::Float(value)) => (expected - *value).abs() < 1e-6,
        _ => false,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use calamine::{open_workbook, Data, Reader, Xlsx};
    use dh_ir::{Cell, CellState, DigitScript, GuardStatus, NumericInfo};
    use std::io::Read;
    use zip::ZipArchive;

    fn writer() -> XlsxWriter {
        XlsxWriter
    }

    fn temporary_path(label: &str) -> PathBuf {
        let unique = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .expect("system clock before Unix epoch")
            .as_nanos();
        std::env::temp_dir().join(format!(
            "dh-writer-{label}-{}-{unique}.xlsx",
            std::process::id()
        ))
    }

    fn temporary_journal(label: &str) -> PathBuf {
        let unique = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .expect("system clock before Unix epoch")
            .as_nanos();
        std::env::temp_dir().join(format!(
            "dh-writer-journal-{label}-{}-{unique}.jsonl",
            std::process::id()
        ))
    }

    fn unlocked_probe(_path: &Path) -> LockStatus {
        LockStatus::Unlocked
    }

    fn locked_probe(_path: &Path) -> LockStatus {
        LockStatus::Locked
    }

    fn make_cell(
        text: &str,
        numeric: Option<NumericInfo>,
        grid_position: Option<(u32, u32)>,
    ) -> Cell {
        Cell {
            state: CellState::Text,
            text: Some(text.to_string()),
            numeric,
            raw: Some(text.to_string()),
            raw_run_texts: vec![text.to_string()],
            bbox: None,
            grid_position,
            rowspan: 1,
            colspan: 1,
            covered_by: None,
            fonts: Vec::new(),
            raw_confidence: None,
            calibrated_confidence: None,
            guard: GuardStatus::Pass,
            provenance: dh_ir::Provenance {
                page: 1,
                region: BBox {
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

    fn make_block(rows: Vec<Vec<Cell>>) -> TableBlock {
        TableBlock {
            page: 1,
            bbox: BBox {
                x0: 0.0,
                y0: 0.0,
                x1: 100.0,
                y1: 50.0,
            },
            rows,
            direction: None,
            segmentation_confidence: None,
            continuation: None,
        }
    }

    fn make_capture(sequence: u64, blocks: Vec<TableBlock>) -> CaptureRecord {
        CaptureRecord {
            id: format!("capture-{sequence}"),
            source_pdf_sha256: "provisional-sha256".to_string(),
            page: 1,
            region: BBox {
                x0: 0.0,
                y0: 0.0,
                x1: 100.0,
                y1: 50.0,
            },
            commit_sequence: sequence,
            blocks,
            unassigned_text: Vec::new(),
        }
    }

    fn numeric_info(value: f64) -> NumericInfo {
        NumericInfo {
            source_text: "30".to_string(),
            canonical: "30".to_string(),
            value,
            decimal_places: 0,
            grouping: false,
            source_digit_script: DigitScript::Latin,
        }
    }

    fn numeric_value(data: &Data) -> Option<f64> {
        match data {
            Data::Float(value) => Some(*value),
            Data::Int(value) => Some(*value as f64),
            _ => None,
        }
    }

    fn sheet_xml(path: &Path) -> String {
        let file = std::fs::File::open(path).expect("open workbook for raw OOXML read");
        let mut archive = ZipArchive::new(file).expect("open workbook as OOXML zip");
        let mut names: Vec<String> = archive
            .file_names()
            .filter_map(|name| name.ok())
            .map(|name| name.to_string())
            .collect();
        names.sort();
        let name = names
            .iter()
            .find(|name| name.contains("worksheets/sheet"))
            .expect("sheet xml entry exists");
        let mut sheet_file = archive.by_name(name).expect("read sheet xml entry");
        let mut buffer = Vec::new();
        sheet_file.read_to_end(&mut buffer).expect("read sheet xml");
        String::from_utf8(buffer).expect("sheet xml is utf-8")
    }

    fn text_capture(sequence: u64, text: &str) -> CaptureRecord {
        CaptureRecord {
            id: format!("capture-{sequence}"),
            source_pdf_sha256: "provisional-sha256".to_string(),
            page: 1,
            region: BBox {
                x0: 0.0,
                y0: 0.0,
                x1: 100.0,
                y1: 50.0,
            },
            commit_sequence: sequence,
            blocks: vec![TableBlock {
                page: 1,
                bbox: BBox {
                    x0: 0.0,
                    y0: 0.0,
                    x1: 100.0,
                    y1: 50.0,
                },
                rows: vec![vec![make_cell(text, None, Some((0, 0)))]],
                direction: None,
                segmentation_confidence: None,
                continuation: None,
            }],
            unassigned_text: Vec::new(),
        }
    }

    fn two_cell_capture(sequence: u64, text_a: &str, value: f64, text_b: &str) -> CaptureRecord {
        CaptureRecord {
            id: format!("capture-{sequence}"),
            source_pdf_sha256: "provisional-sha256".to_string(),
            page: 1,
            region: BBox {
                x0: 0.0,
                y0: 0.0,
                x1: 100.0,
                y1: 50.0,
            },
            commit_sequence: sequence,
            blocks: vec![TableBlock {
                page: 1,
                bbox: BBox {
                    x0: 0.0,
                    y0: 0.0,
                    x1: 100.0,
                    y1: 50.0,
                },
                rows: vec![vec![
                    make_cell(text_a, None, Some((0, 0))),
                    make_cell(text_b, Some(numeric_info(value)), Some((0, 1))),
                ]],
                direction: None,
                segmentation_confidence: None,
                continuation: None,
            }],
            unassigned_text: Vec::new(),
        }
    }

    #[test]
    fn empty_capture_list_writes_valid_blank_workbook() {
        let path = temporary_path("dh-writer-blank");
        let hash = writer()
            .write_capture_list(&[], &path, &scaffold_job_config(), "")
            .expect("empty capture list rebuilds a blank workbook");
        assert!(!hash.is_empty());
        let mut workbook: Xlsx<_> =
            open_workbook(&path).expect("independent reader opens workbook");
        assert!(workbook.sheet_names().iter().any(|n| n == "Sheet1"));
        let range = workbook
            .worksheet_range("Sheet1")
            .expect("independent reader reads blank worksheet");
        assert_eq!(range.used_cells().count(), 0);
    }

    #[test]
    fn rebuild_reproduces_values_and_types_for_captures_with_different_widths() {
        let path = temporary_path("dh-writer-rebuild");
        let first_block = make_block(vec![vec![
            make_cell("Name", None, Some((0, 0))),
            make_cell("Age", None, Some((0, 1))),
            make_cell("Count", Some(numeric_info(30.0)), Some((0, 2))),
        ]]);
        let second_block = make_block(vec![vec![
            make_cell("Item", None, Some((0, 0))),
            make_cell("Qty", None, Some((0, 1))),
            make_cell("Price", None, Some((0, 2))),
            make_cell("Note", None, Some((0, 3))),
            make_cell("Owner", None, Some((0, 4))),
        ]]);
        let captures = vec![
            make_capture(1, vec![first_block]),
            make_capture(2, vec![second_block]),
        ];

        let hash = writer()
            .write_capture_list(&captures, &path, &scaffold_job_config(), "")
            .expect("rebuild with differing capture widths");
        assert!(!hash.is_empty());

        let mut workbook: Xlsx<_> =
            open_workbook(&path).expect("independent reader opens workbook");
        let range = workbook
            .worksheet_range("Sheet1")
            .expect("independent reader reads worksheet");
        assert_eq!(
            range.get_value((0, 0)),
            Some(&Data::String("Name".to_string())),
            "text cell was not rebuilt as text"
        );
        assert_eq!(
            range.get_value((0, 1)),
            Some(&Data::String("Age".to_string())),
            "text cell was not rebuilt as text"
        );
        assert_eq!(
            numeric_value(range.get_value((0, 2)).expect("numeric cell exists")),
            Some(30.0),
            "numeric cell was not rebuilt as a number"
        );
        assert_eq!(
            range.get_value((1, 0)),
            Some(&Data::String("Item".to_string())),
            "second capture must append directly below without a separator row"
        );
        assert_eq!(
            range.get_value((1, 4)),
            Some(&Data::String("Owner".to_string())),
            "differently wide second capture must keep its own columns"
        );
    }

    #[test]
    fn literal_text_and_numeric_rules_match_phase_0_characterization() {
        let path = temporary_path("dh-writer-literal");
        let numeric = NumericInfo {
            source_text: "9007199254740992".to_string(),
            canonical: "9007199254740992".to_string(),
            value: 9_007_199_254_740_992.0,
            decimal_places: 0,
            grouping: false,
            source_digit_script: DigitScript::Latin,
        };
        let block = make_block(vec![vec![
            make_cell("=SUM(1,2)", None, Some((0, 0))),
            make_cell("+1+1", None, Some((0, 1))),
            make_cell("-1", None, Some((0, 2))),
            make_cell("@SUM(A1:A2)", None, Some((0, 3))),
            make_cell("00123", None, Some((0, 4))),
            make_cell("English / فارسی", None, Some((0, 5))),
            make_cell(
                "precision",
                Some(numeric_info(123456789012345680.0)),
                Some((0, 6)),
            ),
        ]]);
        let captures = vec![make_capture(1, vec![block])];

        let hash = writer()
            .write_capture_list(&captures, &path, &scaffold_job_config(), "")
            .expect("rebuild literal characterization workbook");
        assert!(!hash.is_empty());

        let mut workbook: Xlsx<_> =
            open_workbook(&path).expect("independent reader opens workbook");
        let range = workbook
            .worksheet_range("Sheet1")
            .expect("independent reader reads worksheet");
        for (col, expected) in [
            (0u16, "=SUM(1,2)"),
            (1u16, "+1+1"),
            (2u16, "-1"),
            (3u16, "@SUM(A1:A2)"),
            (4u16, "00123"),
            (5u16, "English / فارسی"),
        ] {
            assert_eq!(
                range.get_value((0, u32::from(col))),
                Some(&Data::String(expected.to_string())),
                "literal text {expected} must stay text"
            );
        }
        assert_eq!(
            numeric_value(range.get_value((0, 6)).expect("precision cell exists")),
            Some(123_456_789_012_345_680.0),
            "high-precision numeric must be written as a number"
        );
    }

    #[test]
    fn string_and_grid_excel_limits_are_enforced() {
        let path = temporary_path("dh-writer-limits");

        let long_block = make_block(vec![vec![make_cell(
            &"a".repeat(32_768),
            None,
            Some((0, 0)),
        )]]);
        let error = writer()
            .write_capture_list(
                &[make_capture(1, vec![long_block])],
                &path,
                &scaffold_job_config(),
                "",
            )
            .expect_err("32,768 character strings exceed the Excel limit");
        assert!(
            matches!(error, DhError::Write(_)),
            "unexpected error: {error}"
        );

        let grid_block = make_block(vec![vec![make_cell("too far", None, Some((1_048_576, 0)))]]);
        let error = writer()
            .write_capture_list(
                &[make_capture(1, vec![grid_block])],
                &path,
                &scaffold_job_config(),
                "",
            )
            .expect_err("row 1,048,576 exceeds the Excel grid limit");
        assert!(
            matches!(error, DhError::Write(_)),
            "unexpected error: {error}"
        );
    }

    #[test]
    fn formula_like_and_whitespace_text_is_written_literally() {
        let path = temporary_path("dh-writer-literal-safety");
        let padded = "  leading and trailing  ";
        let with_zwnj = "قبل\u{200c}بعد";
        let values = ["=1+1", "+1+1", "-1", "@SUM(A1:A2)", padded, with_zwnj];
        let block = make_block(vec![values
            .iter()
            .enumerate()
            .map(|(column, value)| make_cell(value, None, Some((0, column as u32))))
            .collect()]);
        writer()
            .write_capture_list(
                &[make_capture(1, vec![block])],
                &path,
                &scaffold_job_config(),
                "",
            )
            .expect("write literal text cells");

        let xml = sheet_xml(&path);
        assert!(
            !xml.contains("<f"),
            "text values must not produce formula elements: {xml}"
        );
        let mut workbook: Xlsx<_> =
            open_workbook(&path).expect("independent reader opens workbook");
        let range = workbook
            .worksheet_range("Sheet1")
            .expect("independent reader reads worksheet");
        for (column, expected) in values.iter().enumerate() {
            assert_eq!(
                range.get_value((0, column as u32)),
                Some(&Data::String((*expected).to_string())),
                "cell {column} must preserve its exact text"
            );
        }
    }

    #[test]
    fn text_limit_accepts_32767_utf16_units_and_rejects_32768() {
        let accepted_path = temporary_path("dh-writer-limit-accepted");
        let accepted = "a".repeat(32_767);
        assert_eq!(accepted.encode_utf16().count(), 32_767);
        writer()
            .write_capture_list(
                &[text_capture(1, &accepted)],
                &accepted_path,
                &scaffold_job_config(),
                "",
            )
            .expect("exactly 32,767 UTF-16 units are accepted");

        let mut workbook: Xlsx<_> =
            open_workbook(&accepted_path).expect("independent reader opens accepted workbook");
        let range = workbook
            .worksheet_range("Sheet1")
            .expect("independent reader reads accepted worksheet");
        assert_eq!(
            range.get_value((0, 0)),
            Some(&Data::String(accepted)),
            "the maximum-length text must be preserved"
        );

        let rejected = "a".repeat(32_768);
        assert_eq!(rejected.encode_utf16().count(), 32_768);
        let error = writer()
            .write_capture_list(
                &[text_capture(1, &rejected)],
                &temporary_path("dh-writer-limit-rejected"),
                &scaffold_job_config(),
                "",
            )
            .expect_err("32,768 UTF-16 units must be rejected");
        assert!(
            matches!(error, DhError::Write(_)),
            "unexpected error: {error}"
        );
    }

    #[test]
    fn surrogate_pairs_are_counted_as_two_utf16_units() {
        let path = temporary_path("dh-writer-surrogate-pairs");
        let accepted = "😀".repeat(16_383);
        assert_eq!(accepted.chars().count(), 16_383);
        assert_eq!(accepted.encode_utf16().count(), 32_766);
        writer()
            .write_capture_list(
                &[text_capture(1, &accepted)],
                &path,
                &scaffold_job_config(),
                "",
            )
            .expect("16,383 surrogate-pair characters fit in 32,766 UTF-16 units");

        let mut workbook: Xlsx<_> =
            open_workbook(&path).expect("independent reader opens emoji workbook");
        let range = workbook
            .worksheet_range("Sheet1")
            .expect("independent reader reads emoji worksheet");
        assert_eq!(
            range.get_value((0, 0)),
            Some(&Data::String(accepted)),
            "emoji text must be preserved exactly"
        );

        let rejected = "😀".repeat(16_384);
        assert_eq!(rejected.chars().count(), 16_384);
        assert_eq!(rejected.encode_utf16().count(), 32_768);
        let error = writer()
            .write_capture_list(
                &[text_capture(1, &rejected)],
                &temporary_path("dh-writer-surrogate-over-limit"),
                &scaffold_job_config(),
                "",
            )
            .expect_err("16,384 surrogate-pair characters exceed the UTF-16 limit");
        assert!(matches!(error, DhError::Write(_)));
    }

    #[test]
    fn external_modification_is_refused_and_file_is_unchanged() {
        let path = temporary_path("dh-writer-external");
        let block = make_block(vec![vec![make_cell("committed", None, Some((0, 0)))]]);
        let hash = writer()
            .write_capture_list(
                &[make_capture(1, vec![block.clone()])],
                &path,
                &scaffold_job_config(),
                "",
            )
            .expect("initial rebuild");
        assert!(!hash.is_empty());
        let committed = std::fs::read(&path).expect("read committed workbook");

        std::fs::write(&path, b"externally modified").expect("tamper workbook");
        let error = writer()
            .write_capture_list(
                &[make_capture(1, vec![block])],
                &path,
                &scaffold_job_config(),
                &hash,
            )
            .expect_err("externally modified workbook must be refused");
        assert!(
            matches!(error, DhError::State(_)),
            "unexpected error: {error}"
        );
        assert_eq!(
            std::fs::read(&path).expect("read workbook"),
            b"externally modified"
        );
    }

    #[test]
    fn failed_rebuild_leaves_last_valid_workbook_intact() {
        let path = temporary_path("dh-writer-failed");
        let block = make_block(vec![vec![make_cell("committed", None, Some((0, 0)))]]);
        let hash = writer()
            .write_capture_list(
                &[make_capture(1, vec![block.clone()])],
                &path,
                &scaffold_job_config(),
                "",
            )
            .expect("initial rebuild");
        let committed = std::fs::read(&path).expect("read committed workbook");

        let long_block = make_block(vec![vec![make_cell(
            &"a".repeat(32_768),
            None,
            Some((0, 0)),
        )]]);
        let error = writer()
            .write_capture_list(
                &[make_capture(1, vec![long_block])],
                &path,
                &scaffold_job_config(),
                &hash,
            )
            .expect_err("over-limit string must fail the rebuild");
        assert!(
            matches!(error, DhError::Write(_)),
            "unexpected error: {error}"
        );
        assert_eq!(std::fs::read(&path).expect("read workbook"), committed);
    }

    #[test]
    fn replace_failure_is_reported_and_temporary_file_is_cleaned() {
        let path = temporary_path("dh-writer-replace");
        let block = make_block(vec![vec![make_cell("committed", None, Some((0, 0)))]]);
        writer()
            .write_capture_list(
                &[make_capture(1, vec![block.clone()])],
                &path,
                &scaffold_job_config(),
                "",
            )
            .expect("initial rebuild");
        let temporary_path = temporary_workbook_path(&path);
        assert!(!temporary_path.exists());

        std::fs::remove_file(&path).expect("remove target for replace-failure simulation");
        std::fs::create_dir(&path).expect("target becomes a directory (rename must fail)");
        let error = writer()
            .write_capture_list(
                &[make_capture(1, vec![block])],
                &path,
                &scaffold_job_config(),
                "",
            )
            .expect_err("rename onto a directory must fail");
        assert!(
            matches!(error, DhError::Write(_)),
            "unexpected error: {error}"
        );
        assert!(
            !temporary_path.exists(),
            "temporary file must be cleaned up"
        );
    }

    #[test]
    fn sequential_appends_with_mixed_dimensions_rebuild_expected_plan() {
        let path = temporary_path("dh-writer-sequential");
        let journal = TransactionJournal::new(&temporary_journal("dh-writer-sequential"));
        let capture_a = make_capture(
            1,
            vec![make_block(vec![vec![
                make_cell("Name", None, Some((0, 0))),
                make_cell("Age", Some(numeric_info(30.0)), Some((0, 1))),
            ]])],
        );
        let capture_b = make_capture(
            2,
            vec![make_block(vec![vec![
                make_cell("Item", None, Some((0, 0))),
                make_cell("Qty", None, Some((0, 1))),
                make_cell("Price", None, Some((0, 2))),
                make_cell("Note", None, Some((0, 3))),
                make_cell("Owner", None, Some((0, 4))),
            ]])],
        );
        let capture_c = make_capture(
            3,
            vec![make_block(vec![vec![make_cell(
                "Only",
                None,
                Some((0, 0)),
            )]])],
        );

        let mut committed_hash = String::new();
        for captures in [
            vec![capture_a.clone()],
            vec![capture_a.clone(), capture_b.clone()],
            vec![capture_a.clone(), capture_b.clone(), capture_c.clone()],
        ] {
            committed_hash = writer()
                .write_capture_list_with_journal(
                    &captures,
                    &path,
                    &committed_hash,
                    &journal,
                    None,
                    Some(&unlocked_probe),
                )
                .expect("journaled sequential commit");
        }

        let xml = sheet_xml(&path);
        assert!(
            xml.contains("<v>"),
            "numeric cell must be written as a real number in raw OOXML: {xml}"
        );
        assert!(
            !xml.contains("<f>"),
            "formula-like text must never become a formula: {xml}"
        );
        let mut workbook: Xlsx<_> =
            open_workbook(&path).expect("independent reader opens workbook");
        let range = workbook
            .worksheet_range("Sheet1")
            .expect("independent reader reads worksheet");
        assert_eq!(
            range.get_value((0, 0)),
            Some(&Data::String("Name".to_string())),
            "capture A text cell was not rebuilt as text"
        );
        assert_eq!(
            numeric_value(range.get_value((0, 1)).expect("numeric cell exists")),
            Some(30.0),
            "capture A numeric cell was not rebuilt as a number"
        );
        assert_eq!(
            range.get_value((1, 0)),
            Some(&Data::String("Item".to_string())),
            "capture B must append directly below capture A without a separator row"
        );
        assert_eq!(
            range.get_value((2, 0)),
            Some(&Data::String("Only".to_string())),
            "capture C must append directly below capture B without a separator row"
        );
    }

    #[test]
    fn failure_before_temp_write_rolls_back_and_restart_keeps_last_valid() {
        let path = temporary_path("dh-writer-fail-before-temp");
        let journal = TransactionJournal::new(&temporary_journal("dh-writer-fail-before-temp"));
        let capture_a = make_capture(
            1,
            vec![make_block(vec![vec![make_cell(
                "Committed",
                None,
                Some((0, 0)),
            )]])],
        );
        let capture_b = make_capture(
            2,
            vec![make_block(vec![vec![make_cell(
                "Pending",
                None,
                Some((0, 0)),
            )]])],
        );
        let committed_hash = writer()
            .write_capture_list_with_journal(
                std::slice::from_ref(&capture_a),
                &path,
                "",
                &journal,
                None,
                Some(&unlocked_probe),
            )
            .expect("initial commit");
        assert_eq!(committed_hash.len(), 64);

        let error = writer()
            .write_capture_list_with_journal(
                &[capture_a.clone(), capture_b.clone()],
                &path,
                &committed_hash,
                &journal,
                Some(FailureStage::BeforeTempWrite),
                Some(&unlocked_probe),
            )
            .expect_err("injected before-temp-write failure");
        assert!(
            matches!(error, DhError::Write(_)),
            "unexpected error: {error}"
        );

        let entries = journal.load().expect("load transaction journal");
        assert_eq!(
            entries.len(),
            2,
            "initial commit plus failed pending commit"
        );
        assert_eq!(entries[1].outcome, JournalOutcome::RolledBack);

        writer()
            .write_capture_list_with_journal(
                &[capture_a],
                &path,
                &committed_hash,
                &journal,
                None,
                Some(&unlocked_probe),
            )
            .expect("restart keeps the last valid committed capture");
    }

    #[test]
    fn failure_after_temp_write_rolls_back_and_restart_keeps_last_valid() {
        let path = temporary_path("dh-writer-fail-after-temp");
        let journal = TransactionJournal::new(&temporary_journal("dh-writer-fail-after-temp"));
        let capture_a = make_capture(
            1,
            vec![make_block(vec![vec![make_cell(
                "Committed",
                None,
                Some((0, 0)),
            )]])],
        );
        let capture_b = make_capture(
            2,
            vec![make_block(vec![vec![make_cell(
                "Pending",
                None,
                Some((0, 0)),
            )]])],
        );
        let committed_hash = writer()
            .write_capture_list_with_journal(
                std::slice::from_ref(&capture_a),
                &path,
                "",
                &journal,
                None,
                Some(&unlocked_probe),
            )
            .expect("initial commit");
        assert_eq!(committed_hash.len(), 64);

        let error = writer()
            .write_capture_list_with_journal(
                &[capture_a.clone(), capture_b.clone()],
                &path,
                &committed_hash,
                &journal,
                Some(FailureStage::AfterTempWrite),
                Some(&unlocked_probe),
            )
            .expect_err("injected after-temp-write failure");
        assert!(
            matches!(error, DhError::Write(_)),
            "unexpected error: {error}"
        );

        let entries = journal.load().expect("load transaction journal");
        assert_eq!(
            entries.len(),
            2,
            "initial commit plus failed pending commit"
        );
        assert_eq!(entries[1].outcome, JournalOutcome::RolledBack);

        writer()
            .write_capture_list_with_journal(
                &[capture_a],
                &path,
                &committed_hash,
                &journal,
                None,
                Some(&unlocked_probe),
            )
            .expect("restart keeps the last valid committed capture");
    }

    #[test]
    fn failure_after_verification_rolls_back_and_restart_keeps_last_valid() {
        let path = temporary_path("dh-writer-fail-after-verify");
        let journal = TransactionJournal::new(&temporary_journal("dh-writer-fail-after-verify"));
        let capture_a = make_capture(
            1,
            vec![make_block(vec![vec![make_cell(
                "Committed",
                None,
                Some((0, 0)),
            )]])],
        );
        let capture_b = make_capture(
            2,
            vec![make_block(vec![vec![make_cell(
                "Pending",
                None,
                Some((0, 0)),
            )]])],
        );
        let committed_hash = writer()
            .write_capture_list_with_journal(
                std::slice::from_ref(&capture_a),
                &path,
                "",
                &journal,
                None,
                Some(&unlocked_probe),
            )
            .expect("initial commit");
        assert_eq!(committed_hash.len(), 64);

        let error = writer()
            .write_capture_list_with_journal(
                &[capture_a.clone(), capture_b.clone()],
                &path,
                &committed_hash,
                &journal,
                Some(FailureStage::AfterVerification),
                Some(&unlocked_probe),
            )
            .expect_err("injected after-verification failure");
        assert!(
            matches!(error, DhError::Write(_)),
            "unexpected error: {error}"
        );

        let entries = journal.load().expect("load transaction journal");
        assert_eq!(
            entries.len(),
            2,
            "initial commit plus failed pending commit"
        );
        assert_eq!(entries[1].outcome, JournalOutcome::RolledBack);
    }

    #[test]
    fn failure_after_replacement_reports_failure_and_restart_adopts_commit() {
        let path = temporary_path("dh-writer-fail-after-replace");
        let journal = TransactionJournal::new(&temporary_journal("dh-writer-fail-after-replace"));
        let capture_a = make_capture(
            1,
            vec![make_block(vec![vec![make_cell(
                "Committed",
                None,
                Some((0, 0)),
            )]])],
        );
        let capture_b = make_capture(
            2,
            vec![make_block(vec![vec![make_cell(
                "Pending",
                None,
                Some((0, 0)),
            )]])],
        );
        let committed_hash = writer()
            .write_capture_list_with_journal(
                std::slice::from_ref(&capture_a),
                &path,
                "",
                &journal,
                None,
                Some(&unlocked_probe),
            )
            .expect("initial commit");
        assert_eq!(committed_hash.len(), 64);

        let error = writer()
            .write_capture_list_with_journal(
                &[capture_a.clone(), capture_b.clone()],
                &path,
                &committed_hash,
                &journal,
                Some(FailureStage::AfterReplacement),
                Some(&unlocked_probe),
            )
            .expect_err("injected after-replacement failure");
        assert!(
            matches!(error, DhError::Write(_)),
            "unexpected error: {error}"
        );

        // After the replacement the journal was already marked committed, so a
        // restart adopts the committed state and replaying the same capture
        // list does not duplicate the capture.
        let entries = journal.load().expect("load transaction journal");
        assert_eq!(
            entries.len(),
            2,
            "initial commit plus committed replacement"
        );
        assert_eq!(entries[1].outcome, JournalOutcome::Committed);
        let adopted_hash = sha256_file(&path).expect("hash adopted workbook");
        writer()
            .write_capture_list_with_journal(
                &[capture_a.clone(), capture_b.clone()],
                &path,
                &adopted_hash,
                &journal,
                None,
                Some(&unlocked_probe),
            )
            .expect("restart adopts the committed replacement without duplication");
    }

    #[test]
    fn failure_before_journal_completion_adopts_commit_on_restart() {
        let path = temporary_path("dh-writer-fail-before-journal");
        let journal = TransactionJournal::new(&temporary_journal("dh-writer-fail-before-journal"));
        let capture_a = make_capture(
            1,
            vec![make_block(vec![vec![make_cell(
                "Committed",
                None,
                Some((0, 0)),
            )]])],
        );
        let capture_b = make_capture(
            2,
            vec![make_block(vec![vec![make_cell(
                "Pending",
                None,
                Some((0, 0)),
            )]])],
        );
        let committed_hash = writer()
            .write_capture_list_with_journal(
                std::slice::from_ref(&capture_a),
                &path,
                "",
                &journal,
                None,
                Some(&unlocked_probe),
            )
            .expect("initial commit");
        assert_eq!(committed_hash.len(), 64);

        let error = writer()
            .write_capture_list_with_journal(
                &[capture_a.clone(), capture_b.clone()],
                &path,
                &committed_hash,
                &journal,
                Some(FailureStage::BeforeJournalCompletion),
                Some(&unlocked_probe),
            )
            .expect_err("injected before-journal-completion failure");
        assert!(
            matches!(error, DhError::Write(_)),
            "unexpected error: {error}"
        );

        let resolution = journal
            .resolve_pending(&path, &committed_hash)
            .expect("resolve pending journal")
            .expect("pending entry resolved");
        assert_eq!(resolution.outcome, JournalOutcome::Committed);
    }

    #[test]
    fn undo_round_trip_rebuilds_previous_workbook_state() {
        let path = temporary_path("dh-writer-undo");
        let journal = TransactionJournal::new(&temporary_journal("dh-writer-undo"));
        let capture_a = make_capture(
            1,
            vec![make_block(vec![vec![make_cell("A", None, Some((0, 0)))]])],
        );
        let capture_b = make_capture(
            2,
            vec![make_block(vec![vec![make_cell("B", None, Some((0, 0)))]])],
        );
        let capture_c = make_capture(
            3,
            vec![make_block(vec![vec![make_cell("C", None, Some((0, 0)))]])],
        );
        let mut committed_hash = String::new();
        for captures in [
            vec![capture_a.clone()],
            vec![capture_a.clone(), capture_b.clone()],
            vec![capture_a.clone(), capture_b.clone(), capture_c.clone()],
        ] {
            committed_hash = writer()
                .write_capture_list_with_journal(
                    &captures,
                    &path,
                    &committed_hash,
                    &journal,
                    None,
                    Some(&unlocked_probe),
                )
                .expect("journaled sequential commit");
        }
        let hash_c = committed_hash;

        let new_hash = writer()
            .undo_capture_list(
                &[capture_a.clone(), capture_b.clone(), capture_c.clone()],
                &path,
                &hash_c,
                &journal,
                None,
                Some(&unlocked_probe),
            )
            .expect("undo of the latest capture rebuilds the previous state");
        assert_eq!(new_hash.len(), 64);
        let mut workbook: Xlsx<_> =
            open_workbook(&path).expect("independent reader opens workbook");
        let range = workbook
            .worksheet_range("Sheet1")
            .expect("independent reader reads worksheet");
        assert_eq!(
            range.get_value((0, 0)),
            Some(&Data::String("A".to_string())),
            "undo must restore the previous workbook state exactly"
        );
        assert_ne!(
            range.get_value((0, 0)),
            Some(&Data::String("C".to_string())),
            "undone capture must be gone"
        );
    }

    #[test]
    fn undo_refused_when_workbook_differs_from_last_committed_hash() {
        let path = temporary_path("dh-writer-undo-refused");
        let journal = TransactionJournal::new(&temporary_journal("dh-writer-undo-refused"));
        let capture_a = make_capture(
            1,
            vec![make_block(vec![vec![make_cell(
                "Committed",
                None,
                Some((0, 0)),
            )]])],
        );
        let hash = writer()
            .write_capture_list_with_journal(
                std::slice::from_ref(&capture_a),
                &path,
                "",
                &journal,
                None,
                Some(&unlocked_probe),
            )
            .expect("initial commit");
        std::fs::write(&path, b"externally modified").expect("tamper workbook");
        let error = writer()
            .undo_capture_list(
                &[capture_a],
                &path,
                &hash,
                &journal,
                None,
                Some(&unlocked_probe),
            )
            .expect_err("undo over an externally modified workbook must be refused");
        assert!(
            matches!(error, DhError::State(_)),
            "unexpected error: {error}"
        );
        assert_eq!(
            std::fs::read(&path).expect("read workbook"),
            b"externally modified"
        );
    }

    #[test]
    fn write_open_probe_failure_refuses_commit_and_undo_without_mutation_then_retry_commits_once() {
        let path = temporary_path("dh-writer-write-open-probe");
        let journal = TransactionJournal::new(&temporary_journal("dh-writer-write-open-probe"));
        let capture_a = make_capture(
            1,
            vec![make_block(vec![vec![make_cell(
                "Committed",
                None,
                Some((0, 0)),
            )]])],
        );
        let capture_b = make_capture(
            2,
            vec![make_block(vec![vec![make_cell(
                "Pending preview",
                None,
                Some((1, 0)),
            )]])],
        );

        let committed_hash = writer()
            .write_capture_list_with_journal(
                std::slice::from_ref(&capture_a),
                &path,
                "",
                &journal,
                None,
                Some(&unlocked_probe),
            )
            .expect("write initial committed workbook");
        let workbook_before = fs::read(&path).expect("read committed workbook");
        let journal_before = journal.load().expect("read committed journal");

        // The test setup supplies the failed write-open result independently
        // of the transaction implementation.
        let error = writer()
            .write_capture_list_with_journal(
                &[capture_a.clone(), capture_b.clone()],
                &path,
                &committed_hash,
                &journal,
                None,
                Some(&locked_probe),
            )
            .expect_err("failed write-open probe must refuse commit");
        assert!(matches!(error, DhError::Write(_)), "unexpected error: {error}");
        assert_eq!(fs::read(&path).expect("re-read workbook"), workbook_before);
        assert_eq!(journal.load().expect("re-read journal"), journal_before);
        assert_eq!(capture_b.blocks[0].rows[0][0].text.as_deref(), Some("Pending preview"));

        let error = writer()
            .undo_capture_list(
                std::slice::from_ref(&capture_a),
                &path,
                &committed_hash,
                &journal,
                None,
                Some(&locked_probe),
            )
            .expect_err("failed write-open probe must refuse undo");
        assert!(matches!(error, DhError::Write(_)), "unexpected error: {error}");
        assert_eq!(fs::read(&path).expect("re-read workbook after refused undo"), workbook_before);
        assert_eq!(journal.load().expect("re-read journal after refused undo"), journal_before);

        let committed_hash = writer()
            .write_capture_list_with_journal(
                &[capture_a, capture_b],
                &path,
                &committed_hash,
                &journal,
                None,
                Some(&unlocked_probe),
            )
            .expect("retry after the probe succeeds");
        assert_eq!(committed_hash.len(), 64);
        let entries = journal.load().expect("load journal after retry");
        assert_eq!(
            entries.iter().filter(|entry| entry.outcome == JournalOutcome::Committed).count(),
            2,
            "retry adds exactly one transaction after the initial commit"
        );
        let mut workbook: Xlsx<_> = open_workbook(&path).expect("independent reader opens workbook");
        let range = workbook
            .worksheet_range("Sheet1")
            .expect("independent reader reads worksheet");
        assert_eq!(
            range.get_value((0, 0)),
            Some(&Data::String("Committed".to_string()))
        );
        assert_eq!(
            range.get_value((1, 0)),
            Some(&Data::String("Pending preview".to_string()))
        );
        assert_eq!(
            range
                .rows()
                .flatten()
                .filter(|cell| **cell == Data::String("Pending preview".to_string()))
                .count(),
            1,
            "retry must not duplicate the pending capture"
        );
    }

    #[test]
    fn excel_and_libreoffice_lock_files_do_not_refuse_a_writable_workbook() {
        let path = temporary_path("dh-writer-lock-files-ignored");
        let journal = TransactionJournal::new(&temporary_journal("dh-writer-lock-files-ignored"));
        let capture = make_capture(
            1,
            vec![make_block(vec![vec![make_cell(
                "Writable despite lock files",
                None,
                Some((0, 0)),
            )]])],
        );
        let file_name = path.file_name().and_then(|name| name.to_str()).expect("workbook filename");
        let excel_lock = path.with_file_name(format!("~${file_name}"));
        let libreoffice_lock = path.with_file_name(format!(".~lock.{file_name}#"));
        fs::write(&excel_lock, b"Excel lock marker").expect("create Excel lock marker");
        fs::write(&libreoffice_lock, b"LibreOffice lock marker").expect("create LibreOffice lock marker");

        assert_eq!(
            XlsxWriter::probe_workbook_lock(&path),
            LockStatus::Unlocked,
            "lock-file presence is not a lock signal when the target is writable"
        );
        writer()
            .write_capture_list_with_journal(
                std::slice::from_ref(&capture),
                &path,
                "",
                &journal,
                None,
                None,
            )
            .expect("lock files alone must not refuse a writable workbook");

        let mut workbook: Xlsx<_> = open_workbook(&path).expect("independent reader opens workbook");
        let range = workbook
            .worksheet_range("Sheet1")
            .expect("independent reader reads worksheet");
        assert_eq!(
            range.get_value((0, 0)),
            Some(&Data::String("Writable despite lock files".to_string()))
        );

        fs::remove_file(excel_lock).expect("remove Excel lock marker");
        fs::remove_file(libreoffice_lock).expect("remove LibreOffice lock marker");
        let _ = fs::remove_file(path);
    }

    #[test]
    fn capture_row_spans_are_computed_per_committed_capture() {
        let capture_a = text_capture(1, "A");
        let capture_b = text_capture(2, "B");
        let captures = vec![capture_a, capture_b];
        let spans = capture_row_spans(&captures);
        assert_eq!(spans, vec![(0, 1), (1, 2)]);
    }

    #[test]
    fn journal_payload_records_the_full_capture_list_before_the_workbook_write() {
        let path = temporary_path("dh-writer-payload");
        let journal = TransactionJournal::new(&temporary_journal("dh-writer-payload"));
        let capture_a = text_capture(1, "Alpha");
        let capture_b = text_capture(2, "Beta");
        let captures = vec![capture_a.clone(), capture_b.clone()];

        writer()
            .write_capture_list_with_journal(
                &captures,
                &path,
                "",
                &journal,
                None,
                Some(&unlocked_probe),
            )
            .expect("journaled commit");

        let entries = journal.load().expect("load journal");
        let entry = entries.last().expect("journal entry");
        assert_eq!(entry.outcome, JournalOutcome::Committed);
        let payload = entry.payload.as_ref().expect("payload is stored");
        assert_eq!(payload.version, JOURNAL_PAYLOAD_VERSION);
        assert_eq!(
            payload.captures, captures,
            "the payload must be the full ordered capture list, including the new capture"
        );
        assert_eq!(
            entry.workbook_sha256_after_commit.as_deref().map(str::len),
            Some(64)
        );
    }

    #[test]
    fn recover_returns_payload_after_crash_before_journal_completion() {
        let path = temporary_path("dh-writer-recover-before-journal");
        let journal =
            TransactionJournal::new(&temporary_journal("dh-writer-recover-before-journal"));
        let capture_a = text_capture(1, "Alpha");
        let capture_b = text_capture(2, "Beta");
        let committed_hash = writer()
            .write_capture_list_with_journal(
                std::slice::from_ref(&capture_a),
                &path,
                "",
                &journal,
                None,
                Some(&unlocked_probe),
            )
            .expect("initial commit");

        writer()
            .write_capture_list_with_journal(
                &[capture_a.clone(), capture_b.clone()],
                &path,
                &committed_hash,
                &journal,
                Some(FailureStage::BeforeJournalCompletion),
                Some(&unlocked_probe),
            )
            .expect_err("injected failure after replacement");

        let recovery = journal
            .recover(&path)
            .expect("recover journal")
            .expect("the on-disk workbook matches a journaled payload");
        assert_eq!(recovery.outcome, JournalOutcome::Committed);
        assert_eq!(
            recovery
                .entry
                .payload
                .as_ref()
                .expect("recovered payload")
                .captures,
            vec![capture_a, capture_b],
            "recovery must return the list that the workbook was rebuilt from"
        );
    }

    #[test]
    fn recover_does_not_adopt_payload_when_workbook_write_never_started() {
        let path = temporary_path("dh-writer-recover-before-temp");
        let journal = TransactionJournal::new(&temporary_journal("dh-writer-recover-before-temp"));
        let capture_a = text_capture(1, "Alpha");
        let capture_b = text_capture(2, "Beta");
        let committed_hash = writer()
            .write_capture_list_with_journal(
                std::slice::from_ref(&capture_a),
                &path,
                "",
                &journal,
                None,
                Some(&unlocked_probe),
            )
            .expect("initial commit");

        writer()
            .write_capture_list_with_journal(
                &[capture_a.clone(), capture_b],
                &path,
                &committed_hash,
                &journal,
                Some(FailureStage::BeforeTempWrite),
                Some(&unlocked_probe),
            )
            .expect_err("injected failure before the workbook write");

        let recovery = journal
            .recover(&path)
            .expect("recover journal")
            .expect("the previous committed payload still matches the workbook");
        assert_eq!(
            recovery
                .entry
                .payload
                .as_ref()
                .expect("recovered payload")
                .captures,
            vec![capture_a],
            "a failed transaction must not replace the previous committed payload"
        );
        let entries = journal.load().expect("load journal");
        assert_eq!(entries[1].outcome, JournalOutcome::RolledBack);
    }

    #[test]
    fn recover_returns_undo_payload_after_crash_before_state_write() {
        let path = temporary_path("dh-writer-recover-undo");
        let journal = TransactionJournal::new(&temporary_journal("dh-writer-recover-undo"));
        let capture_a = text_capture(1, "Alpha");
        let capture_b = text_capture(2, "Beta");
        let mut committed_hash = String::new();
        for captures in [
            vec![capture_a.clone()],
            vec![capture_a.clone(), capture_b.clone()],
        ] {
            committed_hash = writer()
                .write_capture_list_with_journal(
                    &captures,
                    &path,
                    &committed_hash,
                    &journal,
                    None,
                    Some(&unlocked_probe),
                )
                .expect("journaled commit");
        }

        writer()
            .undo_capture_list(
                &[capture_a.clone(), capture_b],
                &path,
                &committed_hash,
                &journal,
                Some(FailureStage::BeforeJournalCompletion),
                Some(&unlocked_probe),
            )
            .expect_err("injected undo failure after replacement");

        let recovery = journal
            .recover(&path)
            .expect("recover journal")
            .expect("the on-disk workbook matches the undo payload");
        assert_eq!(
            recovery
                .entry
                .payload
                .as_ref()
                .expect("recovered payload")
                .captures,
            vec![capture_a],
            "the undo payload must be the remaining capture list"
        );
        assert_eq!(
            recovery.entry.undone_commit_sequence,
            Some(2),
            "the undo entry identifies the removed capture"
        );
    }
    #[test]
    fn partial_capture_cells_are_blank_and_keep_their_destination_row() {
        let path = temporary_path("dh-writer-partial-blank");
        let journal = TransactionJournal::new(&temporary_journal("dh-writer-partial-blank"));
        let mut partial = make_cell("must never be written", None, Some((0, 0)));
        partial.state = CellState::Partial;
        let capture = make_capture(1, vec![make_block(vec![vec![partial]])]);
        let next_capture = make_capture(
            2,
            vec![make_block(vec![vec![make_cell(
                "Next capture",
                None,
                Some((0, 0)),
            )]])],
        );
        let captures = vec![capture, next_capture];
        let spans = capture_row_spans(&captures);
        assert_eq!(spans, vec![(0, 1), (1, 2)]);

        writer()
            .write_capture_list_with_journal(
                &captures,
                &path,
                "",
                &journal,
                None,
                Some(&unlocked_probe),
            )
            .expect("write partial capture through production rebuild path");
        let mut workbook: Xlsx<_> =
            open_workbook(&path).expect("independent calamine reader opens workbook");
        let sheet = workbook
            .worksheet_range("Sheet1")
            .expect("independent calamine reader reads destination worksheet");
        assert!(
            sheet.get_value((0, 0)).is_none()
                || matches!(sheet.get_value((0, 0)), Some(Data::Empty)),
            "partial-cell contents must remain blank in the workbook"
        );
        assert_eq!(
            sheet.get_value((1, 0)),
            Some(&Data::String("Next capture".to_owned())),
            "the next capture must append after the blank partial-cell row"
        );
        let _ = std::fs::remove_file(path);
    }

    #[test]
    fn unassigned_text_is_written_to_review_with_page_coordinates_and_reason() {
        let path = temporary_path("dh-writer-review-unassigned");
        let journal = TransactionJournal::new(&temporary_journal("dh-writer-review-unassigned"));
        let mut capture = make_capture(
            1,
            vec![make_block(vec![vec![make_cell(
                "Assigned",
                None,
                Some((0, 0)),
            )]])],
        );
        capture.unassigned_text.push(UnassignedText {
            run_id: "p1-r9000".to_owned(),
            text: "loose label".to_owned(),
            bbox: BBox {
                x0: 12.5,
                y0: 22.0,
                x1: 48.0,
                y1: 31.5,
            },
            reason: "ambiguous-assignment".to_owned(),
        });

        writer()
            .write_capture_list_with_journal(
                std::slice::from_ref(&capture),
                &path,
                "",
                &journal,
                None,
                Some(&unlocked_probe),
            )
            .expect("write capture and Review record");
        let mut workbook: Xlsx<_> =
            open_workbook(&path).expect("independent calamine reader opens workbook");
        let review = workbook
            .worksheet_range("Review")
            .expect("Review worksheet exists");
        assert!(
            number_cell_matches(review.get_value((1, 0)), 1.0),
            "Review record retains the source page"
        );
        assert_eq!(
            review.get_value((1, 5)),
            Some(&Data::String("ambiguous-assignment".to_owned()))
        );
        assert_eq!(
            review.get_value((1, 6)),
            Some(&Data::String("loose label".to_owned()))
        );
        assert_eq!(
            review.get_value((1, 7)),
            Some(&Data::String("p1-r9000".to_owned()))
        );
        assert!(number_cell_matches(review.get_value((1, 1)), 12.5));
        assert!(number_cell_matches(review.get_value((1, 2)), 22.0));
        assert!(number_cell_matches(review.get_value((1, 3)), 48.0));
        assert!(number_cell_matches(review.get_value((1, 4)), 31.5));
        let _ = std::fs::remove_file(path);
    }
}
