use std::collections::{BTreeMap, BTreeSet};
use std::path::Path;

use anyhow::Context;
use serde::Deserialize;
use serde_json::Value;

use crate::finding::Span;

#[derive(Debug, Default, Deserialize)]
pub struct Manifest {
    pub name: Option<String>,
    #[serde(default)]
    pub private: bool,
    main: Option<String>,
    module: Option<String>,
    types: Option<String>,
    typings: Option<String>,
    bin: Option<Value>,
    exports: Option<Value>,
    workspaces: Option<Workspaces>,
    #[serde(default)]
    dependencies: BTreeMap<String, Value>,
    #[serde(default, rename = "devDependencies")]
    dev_dependencies: BTreeMap<String, Value>,
    #[serde(default, rename = "peerDependencies")]
    peer_dependencies: BTreeMap<String, Value>,
    #[serde(default, rename = "optionalDependencies")]
    optional_dependencies: BTreeMap<String, Value>,
    /// The manifest's text, for locating what it declares.
    #[serde(skip)]
    raw: String,
}

/// The `workspaces` field: an array of patterns (npm, bun) or an object with `packages` (yarn).
#[derive(Debug, Deserialize)]
#[serde(untagged)]
enum Workspaces {
    Patterns(Vec<String>),
    Config {
        #[serde(default)]
        packages: Vec<String>,
    },
}

impl Manifest {
    pub fn read(dir: &Path) -> anyhow::Result<Manifest> {
        let path = dir.join("package.json");
        Self::read_optional(dir)?
            .with_context(|| format!("no package.json found at {}", path.display()))
    }

    pub fn read_optional(dir: &Path) -> anyhow::Result<Option<Manifest>> {
        let path = dir.join("package.json");
        let Ok(raw) = std::fs::read_to_string(&path) else {
            return Ok(None);
        };
        let mut manifest: Manifest =
            serde_json::from_str(&raw).with_context(|| format!("invalid {}", path.display()))?;
        manifest.raw = raw;
        Ok(Some(manifest))
    }

    /// Each declared Dependency once, with where its key is written. A name declared in two
    /// fields is located in the first.
    pub fn dependencies(&self) -> Vec<(String, Span)> {
        // The fields that declare Dependencies, by their manifest key, in reporting order.
        let fields = [
            ("dependencies", &self.dependencies),
            ("devDependencies", &self.dev_dependencies),
            ("peerDependencies", &self.peer_dependencies),
            ("optionalDependencies", &self.optional_dependencies),
        ];
        let mut seen = BTreeSet::new();
        let mut declared = Vec::new();
        for (field, names) in fields {
            for name in names.keys() {
                if seen.insert(name.as_str()) {
                    let span = self.key_span(field, name).unwrap_or(Span::FILE_START);
                    declared.push((name.clone(), span));
                }
            }
        }
        declared
    }

    /// Where `"key":` is written inside the object of top-level `"field":`, found textually
    /// because serde_json keeps no positions.
    fn key_span(&self, field: &str, key: &str) -> Option<Span> {
        let field_at = find_key(&self.raw, field, 0)?;
        let start = find_key(&self.raw, key, field_at)?;
        let before = &self.raw[..start];
        let line_start = before.rfind('\n').map_or(0, |i| i + 1);
        Some(Span {
            start: start as u32,
            end: (start + key.len() + 2) as u32,
            line: before.matches('\n').count() as u32 + 1,
            column: before[line_start..].chars().count() as u32 + 1,
        })
    }

    /// Whether `exports` restricts which subpaths other packages may import.
    pub fn has_exports(&self) -> bool {
        self.exports.is_some()
    }

    pub fn workspace_patterns(&self) -> Vec<String> {
        match &self.workspaces {
            Some(Workspaces::Patterns(patterns) | Workspaces::Config { packages: patterns }) => {
                patterns.clone()
            }
            None => Vec::new(),
        }
    }

    /// Paths of the plain manifest Entry Points, relative to the package directory.
    pub fn entry_paths(&self) -> Vec<String> {
        let mut paths: Vec<String> = [&self.main, &self.module, &self.types, &self.typings]
            .into_iter()
            .flatten()
            .cloned()
            .collect();
        for value in [&self.bin, &self.exports].into_iter().flatten() {
            collect_strings(value, &mut paths);
        }
        // Subpath patterns (`./*`) are expanded by a later provider, not treated as literal paths.
        paths.retain(|p| !p.contains('*'));
        paths
    }
}

/// The byte offset of the first `"key"` at or after `from` that is followed by a colon.
fn find_key(raw: &str, key: &str, from: usize) -> Option<usize> {
    let quoted = format!("\"{key}\"");
    raw[from..].match_indices(&quoted).find_map(|(i, _)| {
        let after = raw[from + i + quoted.len()..].trim_start();
        after.starts_with(':').then_some(from + i)
    })
}

fn collect_strings(value: &Value, out: &mut Vec<String>) {
    match value {
        Value::String(s) => out.push(s.clone()),
        Value::Array(items) => items.iter().for_each(|v| collect_strings(v, out)),
        Value::Object(map) => map.values().for_each(|v| collect_strings(v, out)),
        _ => {}
    }
}
