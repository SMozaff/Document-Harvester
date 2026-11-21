#![allow(dead_code, unused_variables)]

use std::sync::{Arc, Mutex};

use dh_core::Result;

slint::include_modules!();

pub fn new_main_window() -> std::result::Result<MainWindow, slint::PlatformError> {
    MainWindow::new()
}

pub fn run_desktop() -> std::result::Result<(), slint::PlatformError> {
    new_main_window()?.run()
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProgressRow {
    pub file_name: String,
    pub page: u32,
    pub percent: u8,
    pub status: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AlertRow {
    pub cell_address: String,
    pub reason: String,
    pub extracted_text: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ReviewRowView {
    pub sheet: String,
    pub page: u32,
    pub row: u32,
    pub col: u32,
    pub reason: String,
    pub raw_confidence: String,
    pub calibrated_confidence: String,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct UiProjection {
    pub progress: Vec<ProgressRow>,
    pub alerts: Vec<AlertRow>,
    pub review: Vec<ReviewRowView>,
    pub progress_percent: u8,
    pub is_running: bool,
}

#[derive(Clone, Default)]
pub struct SharedUiState {
    inner: Arc<Mutex<UiProjection>>,
}

impl SharedUiState {
    pub fn snapshot(&self) -> UiProjection {
        self.inner
            .lock()
            .map(|state| state.clone())
            .unwrap_or_default()
    }

    pub fn set_progress(&self, rows: Vec<ProgressRow>, percent: u8) {
        if let Ok(mut state) = self.inner.lock() {
            state.progress = rows;
            state.progress_percent = percent;
        }
    }

    pub fn set_alerts(&self, rows: Vec<AlertRow>) {
        if let Ok(mut state) = self.inner.lock() {
            state.alerts = rows;
        }
    }

    pub fn set_review(&self, rows: Vec<ReviewRowView>) {
        if let Ok(mut state) = self.inner.lock() {
            state.review = rows;
        }
    }

    pub fn set_running(&self, running: bool) {
        if let Ok(mut state) = self.inner.lock() {
            state.is_running = running;
        }
    }
}

#[derive(Clone, Default)]
pub struct UiBridge {
    state: SharedUiState,
}

impl UiBridge {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn publish_progress(&self, rows: Vec<ProgressRow>, percent: u8) -> Result<()> {
        self.state.set_progress(rows, percent);
        Ok(())
    }

    pub fn publish_alerts(&self, rows: Vec<AlertRow>) -> Result<()> {
        self.state.set_alerts(rows);
        Ok(())
    }

    pub fn publish_review(&self, rows: Vec<ReviewRowView>) -> Result<()> {
        self.state.set_review(rows);
        Ok(())
    }

    pub fn shared_state(&self) -> SharedUiState {
        self.state.clone()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn projection_updates_are_visible() {
        let bridge = UiBridge::new();
        bridge
            .publish_progress(
                vec![ProgressRow {
                    file_name: " reporting.pdf".into(),
                    page: 1,
                    percent: 10,
                    status: "running".into(),
                }],
                10,
            )
            .unwrap();
        let snapshot = bridge.shared_state().snapshot();
        assert_eq!(snapshot.progress_percent, 10);
        assert_eq!(snapshot.progress.len(), 1);
    }
}
