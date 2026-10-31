mod blind_spots;
pub mod ecosystem;
mod exports;
pub mod finding;
pub mod report;
mod walk;

use std::collections::{BTreeSet, HashMap, HashSet};
use std::path::{Path, PathBuf};

use anyhow::Context;

use blind_spots::BlindSpots;
use ecosystem::{Ecosystem, Resolution, WorkspacePackage, js::JsEcosystem};
use exports::{ParsedFile, UnresolvedImport};
pub use finding::{Confidence, Finding, FindingKind, Reason, ReasonCode, Span};
pub use report::{PackageReport, Report, Summary};

#[derive(Debug, Clone, Default)]
pub struct Options {
    /// Report unused Public API too, instead of counting it as used by code outside the workspace.
    pub include_public_api: bool,
}

/// Scans the workspace at `root` and reports everything it finds unused.
pub fn scan(root: &Path, options: &Options) -> anyhow::Result<Report> {
    let root = root
        .canonicalize()
        .with_context(|| format!("cannot read workspace at {}", root.display()))?;
    let ecosystem = JsEcosystem::load(&root)?;
    let packages = ecosystem.packages();

    let sources: BTreeSet<PathBuf> = walk::files(&root, ecosystem.skipped_dirs())
        .filter(|path| ecosystem.is_source_file(path))
        .collect();
    let mut entry_points = Vec::new();
    // Entry points of published packages, whose exports code outside the workspace may use.
    let mut public_entry_points = HashSet::new();
    for package in packages {
        let entries = ecosystem.entry_points(package)?;
        if !package.private && !options.include_public_api {
            public_entry_points.extend(entries.iter().cloned());
        }
        entry_points.extend(entries);
    }
    let reachable = reachable_files(&ecosystem, entry_points, &sources)?;
    let blind_spots = BlindSpots::find(
        &reachable,
        |file| owning_package(packages, file),
        |file| relative(&root, file),
    );

    let mut reports: Vec<PackageReport> = packages
        .iter()
        .map(|package| PackageReport {
            name: package.name.clone(),
            path: relative(&root, &package.dir),
            findings: Vec::new(),
        })
        .collect();
    for file in sources.iter().filter(|file| !reachable.contains_key(*file)) {
        let Some(owner) = owning_package(packages, file) else {
            continue;
        };
        let path = relative(&root, file);
        reports[owner].findings.push(
            Finding::new(
                FindingKind::UnusedFile,
                &packages[owner].name,
                &path,
                &path,
                Span::FILE_START,
            )
            .lowered_by(blind_spots.reasons(file, owner)),
        );
    }
    for (file, export) in exports::unused(&reachable, &public_entry_points) {
        let Some(owner) = owning_package(packages, file) else {
            continue;
        };
        reports[owner].findings.push(
            Finding::new(
                FindingKind::UnusedExport,
                &packages[owner].name,
                &relative(&root, file),
                &export.name,
                export.span,
            )
            .lowered_by(blind_spots.reasons(file, owner)),
        );
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

/// Source files reachable from the entry points by following imports between the workspace's
/// sources, each parsed once.
fn reachable_files(
    ecosystem: &dyn Ecosystem,
    entry_points: Vec<PathBuf>,
    sources: &BTreeSet<PathBuf>,
) -> anyhow::Result<HashMap<PathBuf, ParsedFile>> {
    let mut reachable: HashMap<PathBuf, ParsedFile> = HashMap::new();
    let mut queue = entry_points;
    while let Some(file) = queue.pop() {
        if reachable.contains_key(&file) || !ecosystem.is_source_file(&file) {
            continue;
        }
        let module = ecosystem.module(&file)?;
        let mut resolved = HashMap::new();
        let mut unresolved = Vec::new();
        for specifier in &module.specifiers {
            match ecosystem.resolve(&file, specifier) {
                Resolution::File(target) if sources.contains(&target) => {
                    if !reachable.contains_key(&target) {
                        queue.push(target.clone());
                    }
                    resolved.insert(specifier.clone(), target);
                }
                Resolution::Unresolved { dir } => unresolved.push(UnresolvedImport {
                    specifier: specifier.clone(),
                    dir,
                }),
                Resolution::File(_) | Resolution::External => {}
            }
        }
        reachable.insert(
            file,
            ParsedFile {
                module,
                resolved,
                unresolved,
            },
        );
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
