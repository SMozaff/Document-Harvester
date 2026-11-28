//! Operation-scoped status records for the application shell.
//!
//! The store keeps one visible record. An unresolved error is sticky until the
//! user explicitly dismisses it; unrelated operation updates cannot hide it.

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Severity {
    Information,
    Success,
    Warning,
    Error,
}

impl Severity {
    pub fn as_code(self) -> i32 {
        match self {
            Self::Information => 0,
            Self::Success => 1,
            Self::Warning => 2,
            Self::Error => 3,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Lifecycle {
    Active,
    Resolved,
    Dismissed,
    Superseded,
}

impl Lifecycle {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Active => "active",
            Self::Resolved => "resolved",
            Self::Dismissed => "dismissed",
            Self::Superseded => "superseded",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RecoveryAction {
    RetryCommit,
}

impl RecoveryAction {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::RetryCommit => "retry-commit",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StatusRecord {
    pub operation_id: String,
    pub source: String,
    pub severity: Severity,
    /// Stable message identity. English display text remains the value for this
    /// English-only shell; catalogue resolution is intentionally not wired.
    pub message_key: String,
    pub message: String,
    pub lifecycle: Lifecycle,
    pub sequence: u64,
    pub recovery_action: Option<RecoveryAction>,
    /// The user-facing blocking explanation is carried in `message`; no
    /// duplicate blocking-note string is maintained in this record.
    pub blocking: bool,
}

impl StatusRecord {
    pub fn new(
        operation_id: impl Into<String>,
        source: impl Into<String>,
        severity: Severity,
        message: impl Into<String>,
        recovery_action: Option<RecoveryAction>,
        blocking: bool,
    ) -> Self {
        let message = message.into();
        Self {
            operation_id: operation_id.into(),
            source: source.into(),
            severity,
            message_key: message.clone(),
            message,
            lifecycle: Lifecycle::Active,
            sequence: 0,
            recovery_action,
            blocking,
        }
    }
}

#[derive(Debug, Default)]
pub struct StatusStore {
    current: Option<StatusRecord>,
    next_sequence: u64,
}

impl StatusStore {
    /// Publish a status and return the record currently presented in the shell.
    ///
    /// An active error is intentionally not superseded by any other operation.
    /// The same failed operation may refresh its own error, but only explicit
    /// dismissal removes the error from the visible channel.
    pub fn publish(&mut self, mut record: StatusRecord) -> Option<StatusRecord> {
        record.sequence = self.take_sequence();
        if let Some(current) = self.current.as_mut() {
            if matches!(current.lifecycle, Lifecycle::Active | Lifecycle::Resolved)
                && current.severity == Severity::Error
            {
                if record.operation_id == current.operation_id && record.severity == Severity::Error
                {
                    current.message = record.message.clone();
                    current.message_key = record.message_key;
                    current.source = record.source;
                    current.sequence = record.sequence;
                    current.blocking = record.blocking;
                    current.recovery_action = record.recovery_action;
                }
                return Some(current.clone());
            }
            current.lifecycle = Lifecycle::Superseded;
        }
        self.current = Some(record);
        self.visible()
    }

    /// Explicitly dismiss the visible record. Only errors expose this action in
    /// the shell, but allowing dismissal of any active record is useful to keep
    /// the store's API straightforward and is covered by its caller's guard.
    pub fn dismiss(&mut self) -> Option<StatusRecord> {
        let sequence = self.take_sequence();
        let current = self.current.as_mut()?;
        if matches!(current.lifecycle, Lifecycle::Active | Lifecycle::Resolved) {
            current.lifecycle = Lifecycle::Dismissed;
            current.sequence = sequence;
        }
        None
    }

    /// Record that a retry succeeded without removing the original error until
    /// the user dismisses its resolved notification.
    pub fn resolve_error(
        &mut self,
        operation_id: &str,
        message: impl Into<String>,
    ) -> Option<StatusRecord> {
        let sequence = self.take_sequence();
        let current = self.current.as_mut()?;
        if current.operation_id != operation_id || current.severity != Severity::Error {
            return None;
        }
        let message = message.into();
        current.message = message.clone();
        current.message_key = message;
        current.lifecycle = Lifecycle::Resolved;
        current.sequence = sequence;
        current.recovery_action = None;
        self.visible()
    }

    pub fn visible(&self) -> Option<StatusRecord> {
        self.current
            .as_ref()
            .filter(|record| matches!(record.lifecycle, Lifecycle::Active | Lifecycle::Resolved))
            .cloned()
    }

    pub fn retry_available(&self) -> bool {
        self.visible()
            .is_some_and(|record| record.recovery_action.is_some())
    }

    fn take_sequence(&mut self) -> u64 {
        self.next_sequence += 1;
        self.next_sequence
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn record(operation_id: &str, severity: Severity, message: &str) -> StatusRecord {
        StatusRecord::new(operation_id, "test", severity, message, None, false)
    }

    #[test]
    fn latest_non_error_status_replaces_the_previous_record() {
        let mut store = StatusStore::default();
        let first = store
            .publish(record("render-1", Severity::Information, "Rendering"))
            .expect("first record visible");
        let second = store
            .publish(record("render-1", Severity::Success, "Rendered"))
            .expect("replacement visible");
        assert_eq!(second.message, "Rendered");
        assert!(second.sequence > first.sequence);
        assert_eq!(second.lifecycle, Lifecycle::Active);
    }

    #[test]
    fn unrelated_success_does_not_clear_an_unresolved_failure() {
        let mut store = StatusStore::default();
        store.publish(record("commit-1", Severity::Error, "Commit failed"));
        let visible = store
            .publish(record("render-2", Severity::Success, "Render complete"))
            .expect("error remains visible");
        assert_eq!(visible.operation_id, "commit-1");
        assert_eq!(visible.message, "Commit failed");
        assert_eq!(visible.lifecycle, Lifecycle::Active);
    }

    #[test]
    fn explicit_dismissal_hides_the_error_and_allows_later_statuses() {
        let mut store = StatusStore::default();
        store.publish(record("undo-1", Severity::Error, "Undo failed"));
        assert!(store.dismiss().is_none());
        assert!(store.visible().is_none());
        let next = store
            .publish(record("navigation", Severity::Information, "Ready"))
            .expect("new status visible after dismissal");
        assert_eq!(next.message, "Ready");
        assert!(next.sequence > 2);
    }

    #[test]
    fn retry_availability_is_derived_from_the_safe_recovery_action() {
        let mut store = StatusStore::default();
        let mut failure = record("commit-1", Severity::Error, "Workbook locked");
        failure.recovery_action = Some(RecoveryAction::RetryCommit);
        store.publish(failure);
        assert!(store.retry_available());
        let visible = store
            .resolve_error("commit-1", "Retry succeeded; dismiss to clear.")
            .expect("error remains after retry succeeds");
        assert_eq!(visible.severity, Severity::Error);
        assert!(!store.retry_available());
    }

    #[test]
    fn retry_success_resolves_but_does_not_hide_error_until_dismissal() {
        let mut store = StatusStore::default();
        let mut failure = record("commit-1", Severity::Error, "Commit failed");
        failure.recovery_action = Some(RecoveryAction::RetryCommit);
        store.publish(failure);
        let resolved = store
            .resolve_error("commit-1", "Retry succeeded; dismiss to clear this notice.")
            .expect("resolved error stays visible");
        assert_eq!(resolved.lifecycle, Lifecycle::Resolved);
        assert_eq!(resolved.severity, Severity::Error);
        assert!(!store.retry_available());
        assert!(store.dismiss().is_none());
        assert!(store.visible().is_none());
    }
}
