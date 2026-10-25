//! All oxc_resolver calls live here, so resolver API churn is contained to one file.

use std::path::{Path, PathBuf};

use oxc_resolver::{ResolveOptions, Resolver as OxcResolver};

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
            ..ResolveOptions::default()
        };
        Resolver {
            inner: OxcResolver::new(options),
        }
    }

    pub fn resolve(&self, from: &Path, specifier: &str) -> Option<PathBuf> {
        let dir = from.parent()?;
        self.inner
            .resolve(dir, specifier)
            .ok()
            .map(|r| r.path().to_path_buf())
    }
}
