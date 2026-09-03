use std::path::Path;

use git2::{ErrorCode, Repository, Status};

use crate::{
    error::{DustError, DustResult},
    models::LockfileStatus,
};

/// Source of Git file status used by the lockfile checker.
pub trait GitStatusProvider {
    /// Returns `None` when `root` is not inside a Git worktree.
    fn status(&self, root: &Path, path: &Path) -> DustResult<Option<LockfileStatus>>;
}

/// Git status provider backed by libgit2.
#[derive(Debug, Default, Clone, Copy)]
pub struct Libgit2StatusProvider;

impl GitStatusProvider for Libgit2StatusProvider {
    fn status(&self, root: &Path, path: &Path) -> DustResult<Option<LockfileStatus>> {
        let repository = match Repository::discover(root) {
            Ok(repository) => repository,
            Err(error) if error.code() == ErrorCode::NotFound => return Ok(None),
            Err(error) => return Err(git_error(error.message())),
        };

        let Some(workdir) = repository.workdir() else {
            // A bare repository has no worktree whose lockfile can be checked.
            return Ok(None);
        };

        let workdir = workdir.canonicalize()?;
        let root = root.canonicalize()?;
        let path = root.join(path);
        let relative_path = path
            .strip_prefix(&workdir)
            .map_err(|_| git_error("lockfile is outside the Git worktree"))?;

        let status = match repository.status_file(relative_path) {
            Ok(status) => status,
            // `git status --porcelain -- <path>` produces no output for paths
            // Git does not report, such as ignored files. The caller has
            // already validated that the lockfile exists, so that is clean for
            // this v1 status model.
            Err(error) if error.code() == ErrorCode::NotFound => Status::CURRENT,
            Err(error) => return Err(git_error(error.message())),
        };

        Ok(Some(status_to_lockfile_status(status)))
    }
}

fn status_to_lockfile_status(status: Status) -> LockfileStatus {
    if status == Status::WT_NEW {
        LockfileStatus::Untracked
    } else if status.is_empty() || status == Status::CURRENT || status == Status::IGNORED {
        LockfileStatus::Clean
    } else {
        LockfileStatus::Modified
    }
}

fn git_error(message: &str) -> DustError {
    DustError::Git(message.to_owned())
}

#[cfg(test)]
mod tests {
    use git2::Status;

    use super::status_to_lockfile_status;
    use crate::models::LockfileStatus;

    #[test]
    fn maps_only_worktree_new_to_untracked() {
        assert_eq!(
            status_to_lockfile_status(Status::WT_NEW),
            LockfileStatus::Untracked
        );
    }

    #[test]
    fn maps_clean_and_ignored_statuses_to_clean() {
        assert_eq!(
            status_to_lockfile_status(Status::CURRENT),
            LockfileStatus::Clean
        );
        assert_eq!(
            status_to_lockfile_status(Status::IGNORED),
            LockfileStatus::Clean
        );
    }

    #[test]
    fn maps_all_other_git_changes_to_modified() {
        assert_eq!(
            status_to_lockfile_status(Status::WT_MODIFIED),
            LockfileStatus::Modified
        );
        assert_eq!(
            status_to_lockfile_status(Status::INDEX_NEW),
            LockfileStatus::Modified
        );
    }
}
