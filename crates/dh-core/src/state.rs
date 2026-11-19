use std::path::{Path, PathBuf};

use crate::{config::JobConfig, triage::PageRoute, Result};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum JobStatus {
    Pending,
    Running,
    Complete,
    FailedParse,
    FailedWrite,
    SkippedCancelled,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct JobState {
    pub input_path: PathBuf,
    pub input_sha256: String,
    pub input_mtime: u64,
    pub input_size: u64,
    pub route: Option<PageRoute>,
    pub output_path: PathBuf,
    pub status: JobStatus,
    pub state_key: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StateKeyInputs {
    pub config: JobConfig,
    pub extractor_name: String,
    pub extractor_version: String,
    pub routing_choice: String,
    pub input_sha256: String,
    pub input_mtime: u64,
    pub input_size: u64,
    pub ocr_inputs: Vec<(String, String)>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RecoveryAction {
    Skip,
    Rerun,
    Abort(String),
}

pub fn compute_state_key(inputs: &StateKeyInputs) -> Result<String> {
    todo!("state::compute_state_key")
}

pub fn load(path: &Path) -> Result<JobState> {
    todo!("state::load")
}

pub fn save(path: &Path, state: &JobState) -> Result<()> {
    todo!("state::save")
}

pub fn recover(state_path: &Path, output_path: &Path) -> Result<RecoveryAction> {
    todo!("state::recover")
}
