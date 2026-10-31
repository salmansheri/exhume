use std::path::Path;

use anyhow::Context;
use serde::Deserialize;
use serde_json::Value;

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
        serde_json::from_str(&raw)
            .map(Some)
            .with_context(|| format!("invalid {}", path.display()))
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

fn collect_strings(value: &Value, out: &mut Vec<String>) {
    match value {
        Value::String(s) => out.push(s.clone()),
        Value::Array(items) => items.iter().for_each(|v| collect_strings(v, out)),
        Value::Object(map) => map.values().for_each(|v| collect_strings(v, out)),
        _ => {}
    }
}
