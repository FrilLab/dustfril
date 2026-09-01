use crate::models::{ExecutableObservation, IntegrityStatus};

pub fn compare(
    previous: &ExecutableObservation,
    current: &ExecutableObservation,
) -> IntegrityStatus {
    if previous.resolved_path != current.resolved_path
        || previous.canonical_path != current.canonical_path
        || previous.symlink_target != current.symlink_target
    {
        return IntegrityStatus::ResolvedPathChanged;
    }

    if previous.sha256 != current.sha256 {
        return IntegrityStatus::ContentChanged;
    }

    IntegrityStatus::Unchanged
}
