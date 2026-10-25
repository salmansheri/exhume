pub mod ecosystem;
pub mod finding;
pub mod report;
mod walk;

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
    let ecosystem = JsEcosystem::load(&root)?;
    let packages = ecosystem.packages();

    let sources: BTreeSet<PathBuf> = walk::files(&root, ecosystem.skipped_dirs())
        .filter(|path| ecosystem.is_source_file(path))
        .collect();
    let mut entry_points = Vec::new();
    for package in packages {
        entry_points.extend(ecosystem.entry_points(package)?);
    }
    let reachable = reachable_files(&ecosystem, entry_points, &sources)?;

    let mut reports: Vec<PackageReport> = packages
        .iter()
        .map(|package| PackageReport {
            name: package.name.clone(),
            path: relative(&root, &package.dir),
            findings: Vec::new(),
        })
        .collect();
    for file in sources.iter().filter(|file| !reachable.contains(*file)) {
        let Some(owner) = owning_package(packages, file) else {
            continue;
        };
        let path = relative(&root, file);
        reports[owner].findings.push(Finding::new(
            FindingKind::UnusedFile,
            &packages[owner].name,
            &path,
            &path,
            Span::FILE_START,
        ));
    }
    Ok(Report::new(reports))
}

/// The innermost Workspace Package whose directory contains `file`.
fn owning_package(packages: &[WorkspacePackage], file: &Path) -> Option<usize> {
    packages
        .iter()
        .enumerate()
        .filter(|(_, package)| file.starts_with(&package.dir))
        .max_by_key(|(_, package)| package.dir.components().count())
        .map(|(index, _)| index)
}

/// Files reachable from the entry points by following imports between the workspace's sources.
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
