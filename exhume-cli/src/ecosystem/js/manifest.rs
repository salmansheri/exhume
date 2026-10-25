use std::path::Path;

use anyhow::Context;
use serde::Deserialize;
use serde_json::Value;

#[derive(Debug, Default, Deserialize)]
pub struct Manifest {
    pub name: Option<String>,
    main: Option<String>,
    module: Option<String>,
    types: Option<String>,
    typings: Option<String>,
    bin: Option<Value>,
    exports: Option<Value>,
}

impl Manifest {
    pub fn read(dir: &Path) -> anyhow::Result<Manifest> {
        let path = dir.join("package.json");
        let raw = std::fs::read_to_string(&path)
            .with_context(|| format!("no package.json found at {}", path.display()))?;
        serde_json::from_str(&raw).with_context(|| format!("invalid {}", path.display()))
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
