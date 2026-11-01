//! Declared Dependencies that no file or script of their Workspace Package uses.

use std::collections::{BTreeSet, HashMap, HashSet};
use std::path::{Path, PathBuf};

use crate::ecosystem::{Dependency, Ecosystem, WorkspacePackage};
use crate::exports::ParsedFile;
use crate::finding::{Reason, ReasonCode};

pub(crate) struct UnusedDependency {
    /// The declaring Workspace Package, by index.
    pub package: usize,
    pub dependency: Dependency,
    /// Package-wide blind spots in unreachable files, which the reachable files' blind spots
    /// don't cover, and unknown script binaries that may be this Dependency's.
    pub reasons: Vec<Reason>,
}

/// Each Workspace Package's unused Dependencies. Every source file a package owns counts,
/// reachable or not: an Unused File still keeps its imports' Dependencies until it is removed.
/// `relative` gives a file's path for display.
pub(crate) fn unused(
    ecosystem: &dyn Ecosystem,
    packages: &[WorkspacePackage],
    sources: &BTreeSet<PathBuf>,
    reachable: &HashMap<PathBuf, ParsedFile>,
    owner: impl Fn(&Path) -> Option<usize>,
    relative: impl Fn(&Path) -> String,
) -> anyhow::Result<Vec<UnusedDependency>> {
    let mut used: Vec<HashSet<String>> = vec![HashSet::new(); packages.len()];
    let mut reasons: Vec<Vec<Reason>> = vec![Vec::new(); packages.len()];
    for file in sources {
        let Some(package) = owner(file) else {
            continue;
        };
        let parsed;
        let specifiers = match reachable.get(file) {
            Some(file) => &file.module.specifiers,
            None => {
                parsed = ecosystem.module(file)?;
                // A non-literal import with no relative start may load any Dependency.
                for opaque in parsed.opaque_imports.iter().filter(|o| o.dir.is_none()) {
                    reasons[package].push(Reason {
                        code: ReasonCode::DynamicImport,
                        detail: Some(format!(
                            "{}:{}:{}",
                            relative(file),
                            opaque.span.line,
                            opaque.span.column
                        )),
                    });
                }
                &parsed.specifiers
            }
        };
        for specifier in specifiers {
            used[package].extend(ecosystem.dependencies_imported_by(specifier));
        }
    }

    let mut unused = Vec::new();
    for (index, package) in packages.iter().enumerate() {
        let script_use = ecosystem.dependencies_run_by_scripts(package);
        used[index].extend(script_use.dependencies);
        // A binary no installed Dependency owns may be any uninstalled one's.
        let bin_guesses: Vec<Reason> = script_use
            .unknown_binaries
            .iter()
            .map(|run| Reason {
                code: ReasonCode::BinGuess,
                detail: Some(format!("{}: {}", run.script, run.binary)),
            })
            .collect();
        for dependency in ecosystem.dependencies(package) {
            if used[index].contains(&dependency.name) {
                continue;
            }
            let mut dependency_reasons = reasons[index].clone();
            if script_use.uninstalled.contains(&dependency.name) {
                dependency_reasons.extend(bin_guesses.iter().cloned());
            }
            unused.push(UnusedDependency {
                package: index,
                dependency,
                reasons: dependency_reasons,
            });
        }
    }
    Ok(unused)
}
