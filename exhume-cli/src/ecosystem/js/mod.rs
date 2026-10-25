mod manifest;
mod parse;
mod resolve;

use std::path::{Path, PathBuf};

use anyhow::Context;

use super::{Ecosystem, WorkspacePackage};

const SOURCE_EXTENSIONS: &[&str] = &["js", "jsx", "mjs", "cjs", "ts", "tsx", "mts", "cts"];

pub struct JsEcosystem {
    resolver: resolve::Resolver,
}

impl JsEcosystem {
    pub fn new() -> Self {
        JsEcosystem {
            resolver: resolve::Resolver::new(),
        }
    }
}

impl Default for JsEcosystem {
    fn default() -> Self {
        Self::new()
    }
}

impl Ecosystem for JsEcosystem {
    fn packages(&self, root: &Path) -> anyhow::Result<Vec<WorkspacePackage>> {
        let manifest = manifest::Manifest::read(root)?;
        let name = manifest.name.unwrap_or_else(|| {
            root.file_name()
                .map(|n| n.to_string_lossy().into_owned())
                .unwrap_or_default()
        });
        Ok(vec![WorkspacePackage {
            name,
            dir: root.to_path_buf(),
        }])
    }

    fn skipped_dirs(&self) -> &'static [&'static str] {
        &["node_modules"]
    }

    fn is_source_file(&self, path: &Path) -> bool {
        let name = path
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or_default();
        // Declaration files are ambient and are never imported, so they can't be reached.
        if [".d.ts", ".d.mts", ".d.cts"]
            .iter()
            .any(|suffix| name.ends_with(suffix))
        {
            return false;
        }
        path.extension()
            .and_then(|e| e.to_str())
            .is_some_and(|e| SOURCE_EXTENSIONS.contains(&e))
    }

    fn entry_points(&self, package: &WorkspacePackage) -> anyhow::Result<Vec<PathBuf>> {
        let manifest = manifest::Manifest::read(&package.dir)?;
        Ok(manifest
            .entry_paths()
            .into_iter()
            .filter_map(|relative| package.dir.join(relative).canonicalize().ok())
            .filter(|path| path.is_file())
            .collect())
    }

    fn imports(&self, file: &Path) -> anyhow::Result<Vec<String>> {
        let source =
            std::fs::read_to_string(file).with_context(|| format!("reading {}", file.display()))?;
        Ok(parse::imports(file, &source))
    }

    fn resolve(&self, from: &Path, specifier: &str) -> Option<PathBuf> {
        self.resolver.resolve(from, specifier)
    }
}
