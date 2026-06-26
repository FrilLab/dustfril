use std::path::Path;

use crate::{
    error::DustResult,
    models::{Ecosystem, ScanResult},
    scanner,
};

pub fn scan(root: &Path, ecosystems: &[Ecosystem]) -> DustResult<ScanResult> {
    scanner::scan(root, ecosystems)
}
