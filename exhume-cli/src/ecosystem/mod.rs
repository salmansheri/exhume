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

    fn resolve(&self, from: &Path, specifier: &str) -> Option<PathBuf>;
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
