mod manifest;
mod parse;
mod resolve;
mod usage;
mod workspace;

use std::collections::HashMap;
use std::path::{Component, Path, PathBuf};

use anyhow::Context;

use super::{Dependency, Ecosystem, Module, Resolution, WorkspacePackage};
use manifest::Manifest;

const SOURCE_EXTENSIONS: &[&str] = &["js", "jsx", "mjs", "cjs", "ts", "tsx", "mts", "cts"];
const SKIPPED_DIRS: &[&str] = &["node_modules"];
/// Node's built-in modules, which can be imported without the `node:` prefix.
const NODE_BUILTINS: &[&str] = &[
    "assert",
    "async_hooks",
    "buffer",
    "child_process",
    "cluster",
    "console",
    "constants",
    "crypto",
    "dgram",
    "diagnostics_channel",
    "dns",
    "domain",
    "events",
    "fs",
    "http",
    "http2",
    "https",
    "inspector",
    "module",
    "net",
    "os",
    "path",
    "perf_hooks",
    "process",
    "punycode",
    "querystring",
    "readline",
    "repl",
    "stream",
    "string_decoder",
    "sys",
    "timers",
    "tls",
    "trace_events",
    "tty",
    "url",
    "util",
    "v8",
    "vm",
    "wasi",
    "worker_threads",
    "zlib",
];

pub struct JsEcosystem {
    packages: Vec<WorkspacePackage>,
    /// Manifests by Workspace Package directory.
    manifests: HashMap<PathBuf, Manifest>,
    /// Workspace Package directories by package name, for imports between packages.
    by_name: HashMap<String, PathBuf>,
    resolver: resolve::Resolver,
}

impl JsEcosystem {
    pub fn load(root: &Path) -> anyhow::Result<Self> {
        let mut packages = Vec::new();
        let mut manifests = HashMap::new();
        let mut by_name = HashMap::new();
        for (package, manifest) in workspace::discover(root)? {
            by_name.insert(package.name.clone(), package.dir.clone());
            manifests.insert(package.dir.clone(), manifest);
            packages.push(package);
        }
        Ok(JsEcosystem {
            packages,
            manifests,
            by_name,
            resolver: resolve::Resolver::new(),
        })
    }

    /// Resolves `@scope/pkg` or `pkg/sub` to a Workspace Package's source without relying on
    /// installed `node_modules` links.
    fn resolve_workspace_package(&self, specifier: &str) -> Option<PathBuf> {
        let (name, subpath) = split_package_specifier(specifier);
        let dir = self.by_name.get(name)?;
        // Self-reference honours the package's `exports`. Only a package without `exports`
        // lets importers reach any file, through `main`, an index file or a relative subpath.
        self.resolver.resolve_from_dir(dir, specifier).or_else(|| {
            let restricted = self.manifests.get(dir).is_some_and(Manifest::has_exports);
            (!restricted)
                .then(|| self.resolver.resolve_from_dir(dir, &format!(".{subpath}")))
                .flatten()
        })
    }
}

impl Ecosystem for JsEcosystem {
    fn packages(&self) -> &[WorkspacePackage] {
        &self.packages
    }

    fn skipped_dirs(&self) -> &'static [&'static str] {
        SKIPPED_DIRS
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
        let manifest = self
            .manifests
            .get(&package.dir)
            .context("package is not part of this workspace")?;
        Ok(manifest
            .entry_paths()
            .into_iter()
            .filter_map(|relative| package.dir.join(relative).canonicalize().ok())
            .filter(|path| path.is_file())
            .collect())
    }

    fn module(&self, file: &Path) -> anyhow::Result<Module> {
        let source =
            std::fs::read_to_string(file).with_context(|| format!("reading {}", file.display()))?;
        Ok(parse::module(file, &source))
    }

    fn resolve(&self, from: &Path, specifier: &str) -> Resolution {
        if let Some(file) = self
            .resolver
            .resolve_from_file(from, specifier)
            .or_else(|| self.resolve_workspace_package(specifier))
        {
            return Resolution::File(file);
        }
        if let Some(dir) = relative_dir(from, specifier) {
            return Resolution::Unresolved { dir: Some(dir) };
        }
        let (name, _) = split_package_specifier(specifier);
        if is_package_name(name) || specifier.contains(':') {
            Resolution::External
        } else {
            // `@/x`, `~/x` and `#x` can't name a package, so they're path aliases exhume
            // couldn't follow.
            Resolution::Unresolved { dir: None }
        }
    }

    fn dependencies(&self, package: &WorkspacePackage) -> Vec<Dependency> {
        let Some(manifest) = self.manifests.get(&package.dir) else {
            return Vec::new();
        };
        manifest
            .dependencies()
            .into_iter()
            .map(|(name, span)| Dependency {
                name,
                manifest: package.dir.join("package.json"),
                span,
            })
            .collect()
    }

    fn dependencies_imported_by(&self, specifier: &str) -> Vec<String> {
        // Built-ins are typed by @types/node; with `node:` they can't mean an npm package.
        if specifier.starts_with("node:") {
            return vec!["@types/node".to_owned()];
        }
        if specifier.starts_with(['.', '/']) || specifier.contains(':') {
            return Vec::new();
        }
        let (name, _) = split_package_specifier(specifier);
        let types = if NODE_BUILTINS.contains(&name) {
            "@types/node".to_owned()
        } else {
            types_package(name)
        };
        if types == name {
            vec![types]
        } else {
            vec![name.to_owned(), types]
        }
    }
}

/// Splits `@scope/pkg/sub/path` into (`@scope/pkg`, `/sub/path`) and `pkg/sub` into (`pkg`, `/sub`).
/// The subpath keeps its leading `/` (or is empty), so `.{subpath}` is a relative specifier.
fn split_package_specifier(specifier: &str) -> (&str, &str) {
    let name_segments = if specifier.starts_with('@') { 2 } else { 1 };
    let end = specifier
        .match_indices('/')
        .nth(name_segments - 1)
        .map_or(specifier.len(), |(i, _)| i);
    specifier.split_at(end)
}

/// The DefinitelyTyped package for `name`: `@types/pkg`, or `@types/scope__pkg` for `@scope/pkg`.
/// An `@types/*` package is its own types.
fn types_package(name: &str) -> String {
    if name.starts_with("@types/") {
        return name.to_owned();
    }
    match name
        .strip_prefix('@')
        .and_then(|scoped| scoped.split_once('/'))
    {
        Some((scope, pkg)) => format!("@types/{scope}__{pkg}"),
        None => format!("@types/{name}"),
    }
}

/// Whether `name` could be an npm package name, `pkg` or `@scope/pkg`.
fn is_package_name(name: &str) -> bool {
    let valid = |part: &str| {
        !part.is_empty()
            && !part.starts_with(['.', '_', '~'])
            && part
                .chars()
                .all(|c| c.is_ascii_alphanumeric() || matches!(c, '-' | '.' | '_' | '~'))
    };
    match name.strip_prefix('@') {
        Some(scoped) => scoped
            .split_once('/')
            .is_some_and(|(scope, pkg)| valid(scope) && valid(pkg)),
        None => valid(name),
    }
}

/// The directory a relative specifier, or a prefix of one such as `./pages/item-`, points into.
/// `None` for a bare specifier.
fn relative_dir(from: &Path, specifier: &str) -> Option<PathBuf> {
    if !(specifier.starts_with("./") || specifier.starts_with("../")) {
        return None;
    }
    let dir = &specifier[..=specifier.rfind('/')?];
    let mut resolved = from.parent()?.to_path_buf();
    for component in Path::new(dir).components() {
        match component {
            Component::ParentDir => {
                resolved.pop();
            }
            Component::Normal(part) => resolved.push(part),
            _ => {}
        }
    }
    Some(resolved)
}
