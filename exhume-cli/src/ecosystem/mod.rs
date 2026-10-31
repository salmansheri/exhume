//! The seam between exhume's core and a language ecosystem (ADR 0001).
//! Nothing outside an ecosystem module may depend on its parser or resolver types.

pub mod js;

use std::path::{Path, PathBuf};

use crate::finding::Span;

#[derive(Debug, Clone)]
pub struct WorkspacePackage {
    pub name: String,
    /// Absolute, canonical directory of the package.
    pub dir: PathBuf,
    /// Never published, so it has no Public API.
    pub private: bool,
}

/// An ecosystem loaded for one workspace, which it has already discovered.
pub trait Ecosystem {
    /// The workspace's packages, the root package (if it has a manifest) included.
    fn packages(&self) -> &[WorkspacePackage];

    /// Directory names that are never walked, whatever the ignore files say.
    fn skipped_dirs(&self) -> &'static [&'static str];

    /// Whether a file can be an Unused File candidate.
    fn is_source_file(&self, path: &Path) -> bool;

    fn entry_points(&self, package: &WorkspacePackage) -> anyhow::Result<Vec<PathBuf>>;

    /// What a source file imports and exports.
    fn module(&self, file: &Path) -> anyhow::Result<Module>;

    fn resolve(&self, from: &Path, specifier: &str) -> Resolution;
}

/// Where a specifier leads.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Resolution {
    /// A file on disk, which may or may not be one of the workspace's sources.
    File(PathBuf),
    /// Outside the workspace, such as an installed or uninstalled package.
    External,
    /// Meant for a file of the workspace, but leads nowhere. Whatever it meant is under `dir`,
    /// or anywhere in the importer's Workspace Package when `dir` is `None`.
    Unresolved { dir: Option<PathBuf> },
}

/// What one source file imports and exports, as its ecosystem understands it.
#[derive(Debug, Clone, Default)]
pub struct Module {
    /// Specifiers of every import and re-export, in source order.
    pub specifiers: Vec<String>,
    pub imports: Vec<Import>,
    /// Exports declared in this file, as opposed to forwarded from another one.
    pub exports: Vec<Export>,
    pub re_exports: Vec<ReExport>,
    pub opaque_imports: Vec<OpaqueImport>,
    /// Specifiers loaded with CommonJS `require()`, whose interop exhume can't follow exactly.
    pub required: Vec<String>,
    /// Lines after an `exhume-ignore-next-line` comment, each of which suppresses one finding.
    pub ignored_lines: Vec<u32>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Import {
    pub specifier: String,
    pub name: ImportedName,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ImportedName {
    /// One export by name; a default import is the export named `default`.
    Name(String),
    /// Every export of the module, as with a namespace import.
    All,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Export {
    pub name: String,
    /// Where the exported name is written.
    pub span: Span,
    /// Marked by a JSDoc `@public` tag, so it counts as used whether or not anything imports it.
    pub tagged_public: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ReExport {
    /// `export { imported as exported } from`, or `export * as exported from` when `imported`
    /// is `All`.
    Named {
        specifier: String,
        imported: ImportedName,
        exported: String,
    },
    /// `export * from`, which forwards every name except `default`.
    Star { specifier: String },
}

/// An import whose specifier can't be read statically, so it may load any file in reach.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OpaqueImport {
    pub span: Span,
    /// The directory every file it can load is under, when its specifier has a static start.
    /// Otherwise it may load any file of its Workspace Package.
    pub dir: Option<PathBuf>,
}
