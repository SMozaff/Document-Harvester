pub use dh_ir::{DhError, Result};

pub fn not_implemented(site: &'static str) -> DhError {
    DhError::NotImplemented(site)
}
