//! Places exhume can't see into, and the findings each one casts doubt on.

use std::collections::HashMap;
use std::path::{Path, PathBuf};

use crate::exports::ParsedFile;
use crate::finding::{Reason, ReasonCode};

/// Which findings a blind spot covers.
enum Scope {
    /// Every finding in files under this directory.
    Dir(PathBuf),
    /// Every finding in this Workspace Package, by index.
    Package(usize),
    /// Every finding in this one file.
    File(PathBuf),
}

struct BlindSpot {
    scope: Scope,
    reason: Reason,
}

impl BlindSpot {
    fn new(scope: Scope, code: ReasonCode, detail: String) -> Self {
        BlindSpot {
            scope,
            reason: Reason {
                code,
                detail: Some(detail),
            },
        }
    }
}

pub(crate) struct BlindSpots(Vec<BlindSpot>);

impl BlindSpots {
    /// The blind spots of the reachable files. `owner` gives a file's Workspace Package and
    /// `relative` its path for display.
    pub fn find(
        files: &HashMap<PathBuf, ParsedFile>,
        owner: impl Fn(&Path) -> Option<usize>,
        relative: impl Fn(&Path) -> String,
    ) -> Self {
        // A blind spot reaches into `dir`, or else anywhere in the file's Workspace Package.
        let scope = |dir: Option<&PathBuf>, file: &Path| match dir {
            Some(dir) => Some(Scope::Dir(dir.clone())),
            None => owner(file).map(Scope::Package),
        };
        let mut spots = Vec::new();
        for (path, file) in files {
            for opaque in &file.module.opaque_imports {
                let Some(scope) = scope(opaque.dir.as_ref(), path) else {
                    continue;
                };
                let location = format!(
                    "{}:{}:{}",
                    relative(path),
                    opaque.span.line,
                    opaque.span.column
                );
                spots.push(BlindSpot::new(scope, ReasonCode::DynamicImport, location));
            }
            for unresolved in &file.unresolved {
                let Some(scope) = scope(unresolved.dir.as_ref(), path) else {
                    continue;
                };
                spots.push(BlindSpot::new(
                    scope,
                    ReasonCode::UnresolvableSpecifier,
                    format!("{} imports {}", relative(path), unresolved.specifier),
                ));
            }
            for specifier in &file.module.required {
                if let Some(target) = file.target(specifier) {
                    spots.push(BlindSpot::new(
                        Scope::File(target.to_path_buf()),
                        ReasonCode::CjsBoundary,
                        format!("required by {}", relative(path)),
                    ));
                }
            }
        }
        BlindSpots(spots)
    }

    /// The reasons to doubt a finding in `file`, which belongs to Workspace Package `owner`.
    pub fn reasons(&self, file: &Path, owner: usize) -> Vec<Reason> {
        let mut reasons: Vec<Reason> = Vec::new();
        for spot in &self.0 {
            let covers = match &spot.scope {
                Scope::Dir(dir) => file.starts_with(dir),
                Scope::Package(package) => *package == owner,
                Scope::File(path) => file == path,
            };
            if covers {
                reasons.push(spot.reason.clone());
            }
        }
        // Blind spots are found in no particular order, but reports must be deterministic.
        reasons.sort();
        reasons.dedup();
        reasons
    }
}
