use std::path::{Path, PathBuf};

/// Directories that are never walked, whatever the ecosystem.
const VCS_DIRS: &[&str] = &[".git", ".hg", ".svn"];

/// Every file under `dir`, honouring the workspace's own ignore files (not those of enclosing
/// directories) whether or not the workspace is a git repo, and never entering `skipped_dirs`.
pub(crate) fn files(
    dir: &Path,
    skipped_dirs: &'static [&'static str],
) -> impl Iterator<Item = PathBuf> {
    ignore::WalkBuilder::new(dir)
        .require_git(false)
        .parents(false)
        .filter_entry(move |entry| {
            !(entry.file_type().is_some_and(|t| t.is_dir())
                && VCS_DIRS
                    .iter()
                    .chain(skipped_dirs)
                    .any(|s| entry.file_name() == *s))
        })
        .build()
        .filter_map(Result::ok)
        .filter(|entry| entry.file_type().is_some_and(|t| t.is_file()))
        .map(|entry| entry.into_path())
}
