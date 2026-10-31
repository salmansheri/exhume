//! Which exports of reachable files something imports.

use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};

use crate::ecosystem::{Export, ImportedName, Module, ReExport};

/// A reachable source file, parsed, with the specifiers that resolved to the workspace's sources.
pub(crate) struct ParsedFile {
    pub module: Module,
    pub resolved: HashMap<String, PathBuf>,
    pub unresolved: Vec<UnresolvedImport>,
}

/// A specifier meant for a file of the workspace that leads nowhere.
pub(crate) struct UnresolvedImport {
    pub specifier: String,
    /// Where the file it meant would be, or `None` for anywhere in the Workspace Package.
    pub dir: Option<PathBuf>,
}

impl ParsedFile {
    /// The workspace source that `specifier` resolved to, if any.
    pub fn target(&self, specifier: &str) -> Option<&Path> {
        self.resolved.get(specifier).map(PathBuf::as_path)
    }
}

/// A request for exports of a file, made by an import.
enum Demand<'a> {
    Name(&'a Path, &'a str),
    All(&'a Path),
}

/// The exports of `files` that nothing imports. Entry points' exports count as used.
///
/// Each import demands a name (or every name) of its target. A demanded name is satisfied by
/// the file's own export of that name, else forwarded by the named re-export that exports it,
/// else forwarded through every `export *`. Demands are deduplicated, so cycles terminate.
pub(crate) fn unused<'a>(
    files: &'a HashMap<PathBuf, ParsedFile>,
    entry_points: &HashSet<PathBuf>,
) -> Vec<(&'a Path, &'a Export)> {
    let mut queue: Vec<Demand<'a>> = Vec::new();
    for (path, file) in files {
        if entry_points.contains(path) {
            queue.push(Demand::All(path));
        }
        for import in &file.module.imports {
            if let Some(target) = file.target(&import.specifier) {
                queue.push(demand(target, &import.name));
            }
        }
    }

    let mut used_names: HashSet<(&Path, &str)> = HashSet::new();
    let mut used_all: HashSet<&Path> = HashSet::new();
    while let Some(next) = queue.pop() {
        match next {
            Demand::Name(path, name) => {
                if !used_names.insert((path, name)) {
                    continue;
                }
                let Some(file) = files.get(path) else {
                    continue;
                };
                if file.module.exports.iter().any(|e| e.name == name) {
                    continue;
                }
                let named = file
                    .module
                    .re_exports
                    .iter()
                    .find_map(|re_export| match re_export {
                        ReExport::Named {
                            specifier,
                            imported,
                            exported,
                        } if exported == name => Some((specifier, imported)),
                        _ => None,
                    });
                if let Some((specifier, imported)) = named {
                    if let Some(target) = file.target(specifier) {
                        queue.push(demand(target, imported));
                    }
                    continue;
                }
                // A name not declared or named here may come through any `export *`.
                if name != "default" {
                    for re_export in &file.module.re_exports {
                        if let ReExport::Star { specifier } = re_export
                            && let Some(target) = file.target(specifier)
                        {
                            queue.push(Demand::Name(target, name));
                        }
                    }
                }
            }
            Demand::All(path) => {
                if !used_all.insert(path) {
                    continue;
                }
                let Some(file) = files.get(path) else {
                    continue;
                };
                for re_export in &file.module.re_exports {
                    match re_export {
                        ReExport::Named {
                            specifier,
                            imported,
                            ..
                        } => {
                            if let Some(target) = file.target(specifier) {
                                queue.push(demand(target, imported));
                            }
                        }
                        // Over-approximates by also using the target's `default`, which is safe:
                        // it can only hide a finding, never invent one.
                        ReExport::Star { specifier } => {
                            if let Some(target) = file.target(specifier) {
                                queue.push(Demand::All(target));
                            }
                        }
                    }
                }
            }
        }
    }

    files
        .iter()
        .filter(|(path, _)| !used_all.contains(path.as_path()))
        .flat_map(|(path, file)| {
            file.module
                .exports
                .iter()
                .map(move |export| (path.as_path(), export))
        })
        .filter(|(path, export)| !used_names.contains(&(*path, export.name.as_str())))
        .collect()
}

fn demand<'a>(target: &'a Path, name: &'a ImportedName) -> Demand<'a> {
    match name {
        ImportedName::Name(name) => Demand::Name(target, name),
        ImportedName::All => Demand::All(target),
    }
}
