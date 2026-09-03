use std::{
    fs::{self, OpenOptions},
    io::{self, Write},
    path::{Path, PathBuf},
    sync::{Mutex, OnceLock},
};

use directories::ProjectDirs;
use serde_json::Value;

use crate::{
    error::{DustError, DustResult},
    models::{INTEGRITY_STATE_VERSION, IntegrityBaseline},
};

static BASELINE_LOCK: OnceLock<Mutex<()>> = OnceLock::new();
static NEXT_TEMP_FILE_ID: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);

/// Access to the versioned local executable-integrity baseline file.
#[derive(Debug, Clone)]
pub struct BaselineStore {
    path: PathBuf,
}

impl BaselineStore {
    pub fn new(path: impl Into<PathBuf>) -> Self {
        Self { path: path.into() }
    }

    pub fn path(&self) -> &Path {
        &self.path
    }

    /// Loads the baseline, returning an explicit error for malformed or old
    /// state instead of silently starting over.
    pub fn load(&self) -> DustResult<IntegrityBaseline> {
        let _guard = baseline_lock()
            .lock()
            .unwrap_or_else(|error| error.into_inner());
        load_unlocked(&self.path)
    }

    /// Atomically replaces the baseline file with a validated v1 state.
    pub fn save(&self, baseline: &IntegrityBaseline) -> DustResult<()> {
        let _guard = baseline_lock()
            .lock()
            .unwrap_or_else(|error| error.into_inner());
        save_unlocked(&self.path, baseline)
    }

    pub(crate) fn update<F, T>(&self, update: F) -> DustResult<T>
    where
        F: FnOnce(&mut IntegrityBaseline) -> DustResult<T>,
    {
        let _guard = baseline_lock()
            .lock()
            .unwrap_or_else(|error| error.into_inner());
        let mut baseline = load_unlocked(&self.path)?;
        let original = baseline.clone();
        let result = update(&mut baseline)?;

        if baseline != original {
            save_unlocked(&self.path, &baseline)?;
        }

        Ok(result)
    }
}

pub fn default_state_path() -> io::Result<PathBuf> {
    let dirs = ProjectDirs::from("dev", "FrilLab", "DustFril")
        .ok_or_else(|| io::Error::other("Failed to determine data directory"))?;
    let data_dir = dirs.data_dir();
    fs::create_dir_all(data_dir)?;
    Ok(data_dir.join("integrity-baseline.json"))
}

fn load_unlocked(path: &Path) -> DustResult<IntegrityBaseline> {
    if !path.exists() {
        return Ok(IntegrityBaseline::default());
    }

    let json = fs::read_to_string(path)?;
    let value: Value = serde_json::from_str(&json).map_err(state_error)?;
    let baseline: IntegrityBaseline = serde_json::from_value(value).map_err(state_error)?;

    if baseline.version != INTEGRITY_STATE_VERSION {
        return Err(DustError::IntegrityState(format!(
            "unsupported executable-integrity state version: {}",
            baseline.version
        )));
    }

    Ok(baseline)
}

fn save_unlocked(path: &Path, baseline: &IntegrityBaseline) -> DustResult<()> {
    if baseline.version != INTEGRITY_STATE_VERSION {
        return Err(DustError::IntegrityState(format!(
            "unsupported executable-integrity state version: {}",
            baseline.version
        )));
    }

    if let Some(parent) = path
        .parent()
        .filter(|parent| !parent.as_os_str().is_empty())
    {
        fs::create_dir_all(parent)?;
    }

    let json = serde_json::to_string_pretty(baseline).map_err(state_error)?;
    let temporary_path = temporary_path(path);
    let write_result = (|| -> io::Result<()> {
        let mut file = OpenOptions::new()
            .create_new(true)
            .write(true)
            .open(&temporary_path)?;
        file.write_all(json.as_bytes())?;
        file.sync_all()?;
        fs::rename(&temporary_path, path)
    })();

    if let Err(error) = write_result {
        let _ = fs::remove_file(&temporary_path);
        return Err(error.into());
    }

    Ok(())
}

fn temporary_path(path: &Path) -> PathBuf {
    let file_name = path
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or("integrity-baseline.json");
    let id = NEXT_TEMP_FILE_ID.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
    path.with_file_name(format!(".{file_name}.{}.{}.tmp", std::process::id(), id))
}

fn state_error(error: serde_json::Error) -> DustError {
    DustError::IntegrityState(error.to_string())
}

fn baseline_lock() -> &'static Mutex<()> {
    BASELINE_LOCK.get_or_init(|| Mutex::new(()))
}
