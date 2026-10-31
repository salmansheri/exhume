//! All oxc parser calls live here and in usage.rs, so oxc API churn is contained to them.

use std::collections::{HashMap, HashSet};
use std::path::Path;

use oxc_allocator::Allocator;
use oxc_ast::Comment;
use oxc_ast::ast::{BindingPattern, Declaration, Program};
use oxc_parser::Parser;
use oxc_span::{GetSpan, SourceType};
use oxc_syntax::module_record::{
    ExportEntry, ExportExportName, ExportImportName, ImportImportName, ModuleRecord,
};

use crate::ecosystem::{Export, Import, ImportedName, Module, OpaqueImport, ReExport};
use crate::finding::Span;

use super::relative_dir;
use super::usage::{self, LoadResult, NameUse};

/// The static imports, re-exports and exports of a file.
pub fn module(path: &Path, source: &str) -> Module {
    let source_type = SourceType::from_path(path).unwrap_or_default();
    let allocator = Allocator::default();
    let parsed = Parser::new(&allocator, source, source_type).parse();
    let record = &parsed.module_record;

    let mut specifiers: Vec<(u32, String)> = record
        .requested_modules
        .iter()
        .filter_map(|(specifier, requests)| {
            let first = requests.iter().map(|r| r.span.start).min()?;
            Some((first, specifier.as_str().to_owned()))
        })
        .collect();
    let file_use = usage::file_use(&parsed.program);
    specifiers.extend(
        file_use
            .loads
            .iter()
            .map(|load| (load.start, load.specifier.clone())),
    );
    specifiers.sort();
    let mut seen = HashSet::new();
    specifiers.retain(|(_, specifier)| seen.insert(specifier.clone()));

    let imports = record
        .import_entries
        .iter()
        .flat_map(|entry| {
            let name_use = file_use.names.get(entry.local_name.name.as_str());
            // A binding that is only exported again is followed through its re-export instead,
            // so the original is used only if something uses the re-export.
            let only_re_exported =
                name_use.is_some_and(|u| u.re_exported && !u.escapes && u.members.is_empty());
            let used = match &entry.import_name {
                _ if only_re_exported => Vec::new(),
                ImportImportName::Name(name) => {
                    vec![ImportedName::Name(name.name.as_str().to_owned())]
                }
                ImportImportName::Default(_) => vec![ImportedName::Name("default".to_owned())],
                ImportImportName::NamespaceObject => used_members(name_use),
            };
            let specifier = entry.module_request.name.as_str();
            used.into_iter().map(move |name| Import {
                specifier: specifier.to_owned(),
                name,
            })
        })
        .chain(file_use.loads.iter().flat_map(|load| {
            let used = match &load.result {
                LoadResult::Discarded => Vec::new(),
                LoadResult::Members(members) => named(members),
                LoadResult::Bound(name) => used_members(file_use.names.get(name)),
                LoadResult::Escapes => vec![ImportedName::All],
            };
            used.into_iter().map(|name| Import {
                specifier: load.specifier.clone(),
                name,
            })
        }))
        .collect();

    // A JSDoc comment is attached to the start of the statement it documents, which is either
    // the export statement or the declaration that `export { name }` exports.
    let tagged_public: HashSet<u32> = parsed
        .program
        .comments
        .iter()
        .filter(|comment| comment.is_jsdoc() && has_public_tag(comment_text(source, comment)))
        .map(|comment| comment.attached_to)
        .collect();
    let declared_at = declaration_starts(&parsed.program);
    let is_tagged_public = |entry: &ExportEntry| {
        let declaration = entry
            .local_name
            .name()
            .and_then(|name| declared_at.get(name.as_str()));
        std::iter::once(&entry.statement_span.start)
            .chain(declaration)
            .any(|start| tagged_public.contains(start))
    };

    let exports = record
        .local_export_entries
        .iter()
        .filter_map(|entry| {
            let (name, span) = export_name(&entry.export_name)?;
            Some(Export {
                name: name.to_owned(),
                span: span_at(source, span),
                tagged_public: is_tagged_public(entry),
            })
        })
        .collect();

    let named = record
        .indirect_export_entries
        .iter()
        .filter_map(|entry| named_re_export(record, entry));
    let stars = record.star_export_entries.iter().filter_map(|entry| {
        Some(ReExport::Star {
            specifier: entry.module_request.as_ref()?.name.as_str().to_owned(),
        })
    });

    Module {
        specifiers: specifiers.into_iter().map(|(_, s)| s).collect(),
        imports,
        exports,
        re_exports: named.chain(stars).collect(),
        opaque_imports: file_use
            .opaque_loads
            .iter()
            .map(|load| OpaqueImport {
                span: span_at(source, load.span),
                dir: load
                    .prefix
                    .as_deref()
                    .and_then(|prefix| relative_dir(path, prefix)),
            })
            .collect(),
        ignored_lines: parsed
            .program
            .comments
            .iter()
            .filter(|comment| comment_text(source, comment).trim() == "exhume-ignore-next-line")
            .map(|comment| span_at(source, comment.span).line + 1)
            .collect(),
        required: file_use
            .loads
            .iter()
            .filter(|load| load.require)
            .map(|load| load.specifier.clone())
            .collect(),
    }
}

/// A re-export, either written as one or made by oxc from `import { a } from "m"; export { a }`.
fn named_re_export(record: &ModuleRecord, entry: &ExportEntry) -> Option<ReExport> {
    let specifier = entry.module_request.as_ref()?.name.as_str().to_owned();
    let (exported, _) = export_name(&entry.export_name)?;
    let imported = match &entry.import_name {
        ExportImportName::Name(name) => {
            // For `import d from "m"; export { d }` oxc names the local binding, not `default`.
            let from_default_import = record.import_entries.iter().any(|import| {
                import.statement_span == entry.statement_span
                    && import.import_name.is_default()
                    && import.local_name.name == name.name
            });
            let imported = if from_default_import {
                "default"
            } else {
                name.name.as_str()
            };
            ImportedName::Name(imported.to_owned())
        }
        ExportImportName::All | ExportImportName::AllButDefault => ImportedName::All,
        ExportImportName::Null => return None,
    };
    Some(ReExport::Named {
        specifier,
        imported,
        exported: exported.to_owned(),
    })
}

/// What a namespace-like binding uses of its module: the members it reads, or everything
/// once it escapes or is exported again. An unused binding uses nothing.
fn used_members(name_use: Option<&NameUse>) -> Vec<ImportedName> {
    match name_use {
        None => Vec::new(),
        Some(name_use) if name_use.escapes || name_use.re_exported => vec![ImportedName::All],
        Some(name_use) => named(&name_use.members),
    }
}

fn named<'a>(members: impl IntoIterator<Item = &'a String>) -> Vec<ImportedName> {
    members
        .into_iter()
        .map(|member| ImportedName::Name(member.clone()))
        .collect()
}

/// The name an export entry exports and where that name is written.
fn export_name<'a>(name: &'a ExportExportName) -> Option<(&'a str, oxc_span::Span)> {
    match name {
        ExportExportName::Name(name) => Some((name.name.as_str(), name.span)),
        ExportExportName::Default(span) => Some(("default", *span)),
        ExportExportName::Null => None,
    }
}

/// Whether JSDoc text has a `@public` tag, as a whole word.
fn has_public_tag(text: &str) -> bool {
    text.match_indices("@public").any(|(i, tag)| {
        !text[i + tag.len()..].starts_with(|c: char| c.is_alphanumeric() || c == '_')
    })
}

/// Where each top-level declaration's statement starts, by declared name.
fn declaration_starts<'a>(program: &Program<'a>) -> HashMap<&'a str, u32> {
    let mut starts = HashMap::new();
    for statement in &program.body {
        let Some(declaration) = statement.as_declaration() else {
            continue;
        };
        let start = statement.span().start;
        match declaration {
            Declaration::VariableDeclaration(variables) => {
                for declarator in &variables.declarations {
                    if let BindingPattern::BindingIdentifier(id) = &declarator.id {
                        starts.insert(id.name.as_str(), start);
                    }
                }
            }
            other => {
                if let Some(id) = other.id() {
                    starts.insert(id.name.as_str(), start);
                }
            }
        }
    }
    starts
}

/// A comment's text without its `//` or `/* */` delimiters.
fn comment_text<'s>(source: &'s str, comment: &Comment) -> &'s str {
    let span = comment.content_span();
    &source[span.start as usize..span.end as usize]
}

/// Converts an oxc byte span into a span with a 1-based line and character column.
fn span_at(source: &str, span: oxc_span::Span) -> Span {
    let before = &source[..span.start as usize];
    let line_start = before.rfind('\n').map_or(0, |i| i + 1);
    Span {
        start: span.start,
        end: span.end,
        line: before.matches('\n').count() as u32 + 1,
        column: before[line_start..].chars().count() as u32 + 1,
    }
}
