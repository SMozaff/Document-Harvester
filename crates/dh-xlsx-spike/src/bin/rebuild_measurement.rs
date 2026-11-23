//! Resource characterization for the proposed rebuild-from-data XLSX flow.
//!
//! Run in release mode, for example:
//!   cargo run --release -p dh-xlsx-spike --bin rebuild_measurement -- 1000,10000,100000 3
//!
//! Evidence only: the numbers are observations on this machine, not a
//! performance requirement or an adoption decision. Peak heap bytes are
//! counted through a global allocator, so they cover allocations made by
//! `rust_xlsxwriter` as well as this binary. They are not process RSS.

use std::{
    alloc::{GlobalAlloc, Layout, System},
    env,
    path::PathBuf,
    sync::atomic::{AtomicUsize, Ordering},
    time::Instant,
};

use rust_xlsxwriter::Workbook;

struct CountingAllocator;

static CURRENT: AtomicUsize = AtomicUsize::new(0);
static PEAK: AtomicUsize = AtomicUsize::new(0);

unsafe impl GlobalAlloc for CountingAllocator {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        let ptr = System.alloc(layout);
        if !ptr.is_null() {
            let size = CURRENT.fetch_add(layout.size(), Ordering::Relaxed) + layout.size();
            PEAK.fetch_max(size, Ordering::Relaxed);
        }
        ptr
    }

    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        CURRENT.fetch_sub(layout.size(), Ordering::Relaxed);
        System.dealloc(ptr, layout);
    }
}

#[global_allocator]
static ALLOCATOR: CountingAllocator = CountingAllocator;

fn linux_peak_rss_kb() -> String {
    #[cfg(target_os = "linux")]
    {
        std::fs::read_to_string("/proc/self/status")
            .ok()
            .and_then(|status| {
                status
                    .lines()
                    .find(|line| line.starts_with("VmHWM:"))
                    .and_then(|line| line.split_whitespace().nth(1))
                    .map(ToString::to_string)
            })
            .unwrap_or_else(|| "unavailable".to_string())
    }
    #[cfg(not(target_os = "linux"))]
    {
        "unavailable".to_string()
    }
}

fn capture_rows(count: u32) -> Vec<(String, String, String, f64)> {
    (0..count)
        .map(|row| {
            (
                format!("capture {row}"),
                "فارسی".to_string(),
                "English / فارسی".to_string(),
                row as f64 / 7.0,
            )
        })
        .collect()
}

fn rebuild(rows: &[(String, String, String, f64)], path: &PathBuf) -> Result<(), String> {
    let mut workbook = Workbook::new();
    let sheet = workbook.add_worksheet();
    sheet
        .set_column_width(0, 22.0)
        .map_err(|error| error.to_string())?;
    sheet
        .set_freeze_panes(1, 0)
        .map_err(|error| error.to_string())?;
    for (row_index, (text, persian, mixed, number)) in rows.iter().enumerate() {
        let row = row_index as u32;
        sheet
            .write_string(row, 0, text)
            .map_err(|error| error.to_string())?;
        sheet
            .write_string(row, 1, persian)
            .map_err(|error| error.to_string())?;
        sheet
            .write_string(row, 2, mixed)
            .map_err(|error| error.to_string())?;
        sheet
            .write_number(row, 3, *number)
            .map_err(|error| error.to_string())?;
    }
    workbook.save(path).map_err(|error| error.to_string())
}

fn main() -> Result<(), String> {
    let args: Vec<String> = env::args().collect();
    let sizes: Vec<u32> = if args.len() > 1 {
        args[1]
            .split(',')
            .map(|part| part.parse::<u32>().map_err(|error| error.to_string()))
            .collect::<Result<_, _>>()?
    } else {
        vec![1_000, 10_000, 100_000]
    };
    let repeats: u32 = if args.len() > 2 {
        args[2].parse::<u32>().map_err(|error| error.to_string())?
    } else {
        3
    };

    let temp_dir = env::temp_dir();
    for rows in sizes {
        let data = capture_rows(rows);
        for repeat in 1..=repeats {
            let path = temp_dir.join(format!(
                "document-harvester-rebuild-measurement-{}-{repeat}.xlsx",
                std::process::id()
            ));
            CURRENT.store(0, Ordering::Relaxed);
            PEAK.store(0, Ordering::Relaxed);
            let started = Instant::now();
            rebuild(&data, &path)?;
            let elapsed = started.elapsed();
            let file_bytes = std::fs::metadata(&path)
                .map_err(|error| error.to_string())?
                .len();
            println!(
                "rows={rows} repeat={repeat}/{repeats} elapsed_ms={} peak_heap_bytes={} file_bytes={file_bytes} process_vmhwm_kb={}",
                elapsed.as_millis(),
                PEAK.load(Ordering::Relaxed),
                linux_peak_rss_kb()
            );
            std::fs::remove_file(path).map_err(|error| error.to_string())?;
        }
    }
    Ok(())
}
