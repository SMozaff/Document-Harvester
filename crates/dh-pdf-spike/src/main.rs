//! Command-line front end for the isolated PDF backend probe.
//!
//! Usage:
//!
//! ```text
//! cargo run -p dh-pdf-spike -- <FILE.pdf> [--render <DPI>] [--max-pages <N>]
//! ```
//!
//! The printed report contains observed backend values only. It is not ground
//! truth and must not be used to claim extraction accuracy.

use std::path::PathBuf;
use std::process::ExitCode;

use dh_pdf_spike::{probe_path, ProbeOptions};

const USAGE: &str = "usage: dh-pdf-spike <FILE.pdf> [--render <DPI>] [--max-pages <N>]";

fn main() -> ExitCode {
    let mut path: Option<PathBuf> = None;
    let mut options = ProbeOptions::default();

    let mut args = std::env::args().skip(1);
    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--render" => match args.next().and_then(|value| value.parse::<u32>().ok()) {
                Some(dpi) => options.render_dpi = Some(dpi),
                None => return usage_error("--render needs a positive integer DPI"),
            },
            "--max-pages" => match args.next().and_then(|value| value.parse::<usize>().ok()) {
                Some(max) => options.max_pages = Some(max),
                None => return usage_error("--max-pages needs a positive integer"),
            },
            "-h" | "--help" => {
                println!("{USAGE}");
                return ExitCode::SUCCESS;
            }
            other if other.starts_with('-') => return usage_error("unknown option"),
            other => {
                if path.is_some() {
                    return usage_error("expected exactly one PDF path");
                }
                path = Some(PathBuf::from(other));
            }
        }
    }

    let Some(path) = path else {
        return usage_error("missing PDF path");
    };

    match probe_path(&path, options) {
        Ok(document) => {
            print!("{}", document.to_report());
            ExitCode::SUCCESS
        }
        Err(error) => {
            // An explicit error is a valid observation for unsupported
            // documents, so it is reported on stderr and not treated as a
            // crash.
            eprintln!("backend error for {path:?}: {error}");
            ExitCode::from(1)
        }
    }
}

fn usage_error(message: &str) -> ExitCode {
    eprintln!("{message}");
    eprintln!("{USAGE}");
    ExitCode::from(2)
}
