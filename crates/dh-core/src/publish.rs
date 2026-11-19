use std::path::{Path, PathBuf};

use crate::{state::JobState, Result};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Platform {
    Linux,
    Windows,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PublishPaths {
    pub temp: PathBuf,
    pub output: PathBuf,
    pub done: PathBuf,
    pub state: PathBuf,
}

pub fn atomic_write_file(source: &Path, destination: &Path, platform: Platform) -> Result<()> {
    todo!("publish::atomic_write_file")
}

pub fn sync_directory(path: &Path, platform: Platform) -> Result<()> {
    todo!("publish::sync_directory")
}

pub fn publish(paths: &PublishPaths, state: &JobState, platform: Platform) -> Result<()> {
    todo!("publish::publish")
}
