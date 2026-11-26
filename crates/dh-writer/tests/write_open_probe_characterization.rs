//! CI-only characterization of the D19 workbook write-open probe.
//!
//! Owner decision D19 (`docs/DECISIONS.md`) treats a workbook as locked only
//! when the write-open probe fails; lock files are not checked. The production
//! commit and undo paths refuse through the default write-open probe before any
//! journal, workbook or private-state mutation.
//!
//! This test is a characterization record, not a fix and not an assertion about
//! the observed outcome. It creates a workbook with the production writer, starts
//! LibreOffice headless with that workbook open, drives the real commit path
//! while the spreadsheet process holds the file, records whether the write-open
//! probe reported the workbook as locked or writable, stops LibreOffice, and
//! re-reads the workbook with the independent calamine reader.
//!
//! The assertions in this test are limited to: the characterization ran, the
//! workbook remained readable with calamine afterwards, and the write-open probe
//! returned one of the two D19 lock states.

use std::{
    env, fs,
    path::Path,
    process::{self, Child, Command, Stdio},
    thread,
    time::{Duration, Instant},
};

use calamine::{open_workbook, Reader, Xlsx};
use dh_writer::{file_sha256, TransactionJournal, XlsxWriter};

/// D19 write-open probe verdicts.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ProbeVerdict {
    /// The write-open probe opened the workbook for writing: writable.
    Writable,
    /// The write-open probe failed: locked.
    Locked,
}

impl ProbeVerdict {
    fn label(self) -> &'static str {
        match self {
            ProbeVerdict::Writable => "writable",
            ProbeVerdict::Locked => "locked",
        }
    }
}

/// Owns the LibreOffice child process so it is stopped on every exit path.
struct LibreOfficeProcess {
    child: Child,
}

impl LibreOfficeProcess {
    fn start(workbook: &Path) -> Self {
        let child = Command::new("libreoffice")
            .arg("--headless")
            .arg(workbook)
            .stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .spawn()
            .expect("spawn LibreOffice headless with the characterization workbook open");
        Self { child }
    }

    fn is_running(&mut self) -> bool {
        matches!(self.child.try_wait(), Ok(None))
    }
}

impl Drop for LibreOfficeProcess {
    fn drop(&mut self) {
        // Kill (SIGKILL) and reap so LibreOffice is stopped on every exit
        // path, including panic unwind.
        let _ = self.child.kill();
        let _ = self.child.wait();
    }
}

fn lock_characterization_enabled() -> bool {
    env::var_os("DH_LOCK_CHARACTERIZATION")
        .is_some_and(|value| value.to_str().is_some_and(|value| value == "1"))
}

fn record_probe_result(verdict: ProbeVerdict, workbook: &Path) {
    let line = format!(
        "[D19 write-open probe characterization] workbook '{}' reported: {}",
        workbook.display(),
        verdict.label()
    );
    println!("{line}");
    let _ = fs::write(
        env::temp_dir().join(format!("dh-lock-characterization-{}.txt", process::id())),
        format!("{line}\n"),
    );
}

#[test]
fn libreoffice_write_open_probe_characterization() {
    if !lock_characterization_enabled() {
        // CI-only characterization: skip on developer machines unless opted in.
        return;
    }

    let directory = env::temp_dir().join(format!("dh-lock-characterization-{}", process::id()));
    fs::create_dir_all(&directory).expect("create characterization workbook directory");
    let workbook_path = directory.join("characterization-workbook.xlsx");
    let journal = TransactionJournal::new(&directory.join("characterization-journal.jsonl"));
    let writer = XlsxWriter;

    writer
        .create_blank(&workbook_path)
        .expect("production writer creates the characterization workbook");
    let committed_hash = file_sha256(&workbook_path).expect("hash the characterization workbook");

    let mut libreoffice = LibreOfficeProcess::start(&workbook_path);
    let startup_deadline = Instant::now() + Duration::from_secs(60);
    while libreoffice.is_running() && Instant::now() < startup_deadline {
        thread::sleep(Duration::from_millis(250));
    }

    let verdict = match writer.write_capture_list_with_journal(
        &[],
        &workbook_path,
        &committed_hash,
        &journal,
        None,
        None,
    ) {
        Ok(_) => Some(ProbeVerdict::Writable),
        Err(error) => {
            if error.to_string().contains("is locked") {
                Some(ProbeVerdict::Locked)
            } else {
                None
            }
        }
    }
    .expect("the D19 write-open probe must have returned one of the two lock states");

    record_probe_result(verdict, &workbook_path);
    drop(libreoffice);

    let mut workbook: Xlsx<_> = open_workbook(&workbook_path)
        .expect("independent calamine reader reopens the workbook after the LibreOffice probe");
    assert!(
        workbook.sheet_names().iter().any(|name| name == "Sheet1"),
        "independent reader must still find the destination worksheet after the LibreOffice probe"
    );
    let _range = workbook
        .worksheet_range("Sheet1")
        .expect("independent reader reads the destination worksheet after the LibreOffice probe");

    let _ = fs::remove_dir_all(&directory);
}
