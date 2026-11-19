use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

use dh_core::{config::JobConfig, state::JobState, Result};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BatchOptions {
    pub input_dir: PathBuf,
    pub output_dir: PathBuf,
    pub triage_only: bool,
    pub force: bool,
}

#[derive(Debug, Clone, Default)]
pub struct CancellationToken {
    flag: Arc<AtomicBool>,
}

impl CancellationToken {
    pub fn is_cancelled(&self) -> bool {
        self.flag.load(Ordering::Relaxed)
    }

    pub fn cancel(&self) {
        self.flag.store(true, Ordering::Relaxed);
    }
}

pub fn available_lane_count() -> usize {
    std::thread::available_parallelism()
        .map(|value| value.get())
        .unwrap_or(1)
}

pub fn run_batch(
    options: &BatchOptions,
    config: &JobConfig,
    cancellation: &CancellationToken,
) -> Result<Vec<JobState>> {
    todo!("batch::run_batch")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cancellation_is_cooperative() {
        let token = CancellationToken::default();
        assert!(!token.is_cancelled());
        token.cancel();
        assert!(token.is_cancelled());
    }

    #[test]
    fn lane_count_is_positive() {
        assert!(available_lane_count() >= 1);
    }
}
