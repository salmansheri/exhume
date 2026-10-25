//! All oxc_resolver calls live here, so resolver API churn is contained to one file.

use std::path::{Path, PathBuf};

use oxc_resolver::{ResolveOptions, Resolver as OxcResolver, TsconfigDiscovery};

pub struct Resolver {
    inner: OxcResolver,
}

impl Resolver {
    pub fn new() -> Self {
        let options = ResolveOptions {
            extensions: [
                ".ts", ".tsx", ".mts", ".cts", ".js", ".jsx", ".mjs", ".cjs", ".json",
            ]
            .map(String::from)
            .to_vec(),
            // TypeScript lets `./foo.js` refer to `./foo.ts`.
            extension_alias: vec![
                (
                    ".js".into(),
                    [".ts", ".tsx", ".js", ".jsx"].map(String::from).to_vec(),
                ),
                (".mjs".into(), [".mts", ".mjs"].map(String::from).to_vec()),
                (".cjs".into(), [".cts", ".cjs"].map(String::from).to_vec()),
            ],
            condition_names: ["import", "require", "node", "default"]
                .map(String::from)
                .to_vec(),
            // Each file's nearest tsconfig supplies its `paths` aliases.
            tsconfig: Some(TsconfigDiscovery::Auto),
            ..ResolveOptions::default()
        };
        Resolver {
            inner: OxcResolver::new(options),
        }
    }

    /// Resolves `specifier` as imported by `file`, applying that file's tsconfig `paths`.
    pub fn resolve_from(&self, file: &Path, specifier: &str) -> Option<PathBuf> {
        self.inner
            .resolve_file(file, specifier)
            .ok()
            .map(|r| r.path().to_path_buf())
    }

    /// Resolves `specifier` as if it were imported by a file in `dir`, without tsconfig `paths`.
    pub fn resolve_in(&self, dir: &Path, specifier: &str) -> Option<PathBuf> {
        self.inner
            .resolve(dir, specifier)
            .ok()
            .map(|r| r.path().to_path_buf())
    }
}
