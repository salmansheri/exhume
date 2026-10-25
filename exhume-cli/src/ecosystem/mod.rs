//! The seam between exhume's core and a language ecosystem (ADR 0001).
//! Nothing outside an ecosystem module may depend on its parser or resolver types.

pub mod js;

use std::path::{Path, PathBuf};

#[derive(Debug, Clone)]
pub struct WorkspacePackage {
    pub name: String,
    /// Absolute, canonical directory of the package.
    pub dir: PathBuf,
}

/// An ecosystem loaded for one workspace, which it has already discovered.
pub trait Ecosystem {
    /// The workspace's packages, the root package (if it has a manifest) included.
    fn packages(&self) -> &[WorkspacePackage];

    /// Directory names that are never walked, whatever the ignore files say.
    fn skipped_dirs(&self) -> &'static [&'static str];

    /// Whether a file can be an Unused File candidate.
    fn is_source_file(&self, path: &Path) -> bool;

    fn entry_points(&self, package: &WorkspacePackage) -> anyhow::Result<Vec<PathBuf>>;

    /// The specifiers a file imports, in source order.
    fn imports(&self, file: &Path) -> anyhow::Result<Vec<String>>;

    fn resolve(&self, from: &Path, specifier: &str) -> Option<PathBuf>;
}
