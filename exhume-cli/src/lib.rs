pub mod ecosystem;
pub mod finding;
pub mod report;

use std::collections::{BTreeSet, HashSet};
use std::path::{Path, PathBuf};

use anyhow::Context;

use ecosystem::{Ecosystem, WorkspacePackage, js::JsEcosystem};
pub use finding::{Confidence, Finding, FindingKind, Reason, ReasonCode, Span};
pub use report::{PackageReport, Report, Summary};

#[derive(Debug, Clone, Default)]
pub struct Options {}

/// Scans the workspace at `root` and reports everything it finds unused.
pub fn scan(root: &Path, _options: &Options) -> anyhow::Result<Report> {
    let root = root
        .canonicalize()
        .with_context(|| format!("cannot read workspace at {}", root.display()))?;
    let ecosystem = JsEcosystem::new();
    let packages = ecosystem
        .packages(&root)?
        .iter()
        .map(|package| scan_package(&ecosystem, &root, package))
        .collect::<anyhow::Result<_>>()?;
    Ok(Report::new(packages))
}

fn scan_package(
    ecosystem: &dyn Ecosystem,
    root: &Path,
    package: &WorkspacePackage,
) -> anyhow::Result<PackageReport> {
    let sources = source_files(ecosystem, &package.dir);
    let reachable = reachable_files(ecosystem, ecosystem.entry_points(package)?, &sources)?;
    let findings = sources
        .iter()
        .filter(|file| !reachable.contains(*file))
        .map(|file| {
            let path = relative(root, file);
            Finding::new(
                FindingKind::UnusedFile,
                &package.name,
                &path,
                &path,
                Span::FILE_START,
            )
        })
        .collect();
    Ok(PackageReport {
        name: package.name.clone(),
        path: relative(root, &package.dir),
        findings,
    })
}

/// Directories that are never walked, whatever the ecosystem.
const VCS_DIRS: &[&str] = &[".git", ".hg", ".svn"];

/// Every source file in `dir`, honouring the workspace's own ignore files (not those of enclosing
/// directories) whether or not the workspace is a git repo.
fn source_files(ecosystem: &dyn Ecosystem, dir: &Path) -> BTreeSet<PathBuf> {
    let skipped = ecosystem.skipped_dirs();
    ignore::WalkBuilder::new(dir)
        .require_git(false)
        .parents(false)
        .filter_entry(move |entry| {
            !(entry.file_type().is_some_and(|t| t.is_dir())
                && VCS_DIRS
                    .iter()
                    .chain(skipped)
                    .any(|s| entry.file_name() == *s))
        })
        .build()
        .filter_map(Result::ok)
        .filter(|entry| entry.file_type().is_some_and(|t| t.is_file()))
        .map(|entry| entry.into_path())
        .filter(|path| ecosystem.is_source_file(path))
        .collect()
}

/// Files reachable from the entry points by following imports between the package's own sources.
fn reachable_files(
    ecosystem: &dyn Ecosystem,
    entry_points: Vec<PathBuf>,
    sources: &BTreeSet<PathBuf>,
) -> anyhow::Result<HashSet<PathBuf>> {
    let mut reachable: HashSet<PathBuf> = HashSet::new();
    let mut queue = entry_points;
    while let Some(file) = queue.pop() {
        if !reachable.insert(file.clone()) || !ecosystem.is_source_file(&file) {
            continue;
        }
        for specifier in ecosystem.imports(&file)? {
            if let Some(target) = ecosystem.resolve(&file, &specifier)
                && sources.contains(&target)
                && !reachable.contains(&target)
            {
                queue.push(target);
            }
        }
    }
    Ok(reachable)
}

fn relative(root: &Path, path: &Path) -> String {
    let rel = path.strip_prefix(root).unwrap_or(path);
    let text = rel.to_string_lossy().replace('\\', "/");
    if text.is_empty() {
        ".".to_owned()
    } else {
        text
    }
}
