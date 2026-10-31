//! The AST half of parsing: how each name is used, and the `require()` and `import()` calls.
//! Kept beside parse.rs so oxc AST churn is contained to this directory.

use std::collections::{BTreeSet, HashMap};

use oxc_ast::AstKind;
use oxc_ast::ast::{Argument, BindingPattern, Expression, Program, VariableDeclarator};
use oxc_ast_visit::Visit;
use oxc_span::GetSpan;
use oxc_syntax::operator::BinaryOperator;

/// How a name is used across a file. Names are matched without regard to scope, so a shadowing
/// local can only add uses, which hides findings rather than inventing them.
#[derive(Debug, Default)]
pub struct NameUse {
    /// Members read with a static key, as in `ns.a` or the type `ns.A`.
    pub members: BTreeSet<String>,
    /// Whether the name is exported again, as in `export { name }`.
    pub re_exported: bool,
    /// Whether the name is used any other way: passed, spread, indexed with a computed key.
    pub escapes: bool,
}

/// A `require()` or `import()` call with a literal specifier.
#[derive(Debug)]
pub struct Load {
    pub specifier: String,
    pub start: u32,
    /// Whether it was `require()` rather than `import()`.
    pub require: bool,
    pub result: LoadResult,
}

/// What happens to the module object a load returns.
#[derive(Debug)]
pub enum LoadResult {
    /// Thrown away, as in `require("./polyfill")`.
    Discarded,
    /// Only these members are read, as in `require("x").a` or `const { a } = await import("x")`.
    Members(Vec<String>),
    /// Bound to a name, whose uses decide which members are read.
    Bound(String),
    /// Used some other way, so every export may be read.
    Escapes,
}

/// A `require()` or `import()` call whose specifier exhume can't read.
#[derive(Debug)]
pub struct OpaqueLoad {
    pub span: oxc_span::Span,
    /// The specifier's static start, as `./pages/` in `` import(`./pages/${name}`) ``.
    pub prefix: Option<String>,
}

#[derive(Debug, Default)]
pub struct FileUse {
    pub names: HashMap<String, NameUse>,
    pub loads: Vec<Load>,
    pub opaque_loads: Vec<OpaqueLoad>,
}

pub fn file_use(program: &Program) -> FileUse {
    let mut visitor = Visitor::default();
    visitor.visit_program(program);
    visitor.file_use
}

#[derive(Default)]
struct Visitor<'a> {
    /// The nodes enclosing the one being visited, innermost last.
    ancestors: Vec<AstKind<'a>>,
    file_use: FileUse,
}

impl<'a> Visitor<'a> {
    fn load(&mut self, source: &Expression<'a>, span: oxc_span::Span, require: bool) {
        let Some(specifier) = literal(source) else {
            self.file_use.opaque_loads.push(OpaqueLoad {
                span,
                prefix: static_prefix(source),
            });
            return;
        };
        let result = self.load_result(span, require);
        self.file_use.loads.push(Load {
            specifier,
            start: span.start,
            require,
            result,
        });
    }

    /// Follows the call at `span` outwards, through `await` and other wrappers, to its use.
    /// `import()` yields a promise, so only once awaited is its use a use of the module.
    fn load_result(&self, mut span: oxc_span::Span, require: bool) -> LoadResult {
        let mut is_module = require;
        for ancestor in self.ancestors.iter().rev() {
            match ancestor {
                AstKind::ParenthesizedExpression(e) => span = e.span,
                AstKind::AwaitExpression(e) => {
                    span = e.span;
                    is_module = true;
                }
                AstKind::TSAsExpression(e) => span = e.span,
                AstKind::TSNonNullExpression(e) => span = e.span,
                AstKind::ExpressionStatement(_) => return LoadResult::Discarded,
                // `.then(m => ...)` and other uses of the promise aren't followed.
                _ if !is_module => return LoadResult::Escapes,
                AstKind::StaticMemberExpression(member) if member.object.span() == span => {
                    return LoadResult::Members(vec![member.property.name.as_str().to_owned()]);
                }
                AstKind::VariableDeclarator(declarator) if initializes(declarator, span) => {
                    return match &declarator.id {
                        BindingPattern::BindingIdentifier(id) => {
                            LoadResult::Bound(id.name.as_str().to_owned())
                        }
                        pattern => {
                            destructured(pattern).map_or(LoadResult::Escapes, LoadResult::Members)
                        }
                    };
                }
                _ => return LoadResult::Escapes,
            }
        }
        LoadResult::Escapes
    }
}

fn initializes(declarator: &VariableDeclarator, span: oxc_span::Span) -> bool {
    declarator
        .init
        .as_ref()
        .is_some_and(|init| init.span() == span)
}

/// The keys an object pattern such as `{ a, b: c }` takes, when every key is static and there
/// is no rest element.
fn destructured(pattern: &BindingPattern) -> Option<Vec<String>> {
    let BindingPattern::ObjectPattern(pattern) = pattern else {
        return None;
    };
    if pattern.rest.is_some() {
        return None;
    }
    pattern
        .properties
        .iter()
        .map(|property| {
            let name = property.key.static_name()?;
            (!property.computed).then(|| name.as_str().map(str::to_owned))?
        })
        .collect()
}

/// The text of a specifier written as a string or a template literal with no substitutions.
fn literal(source: &Expression) -> Option<String> {
    match source {
        Expression::StringLiteral(s) => s.value.as_str().map(str::to_owned),
        Expression::TemplateLiteral(t) if t.expressions.is_empty() => {
            t.quasis.first()?.value.cooked?.as_str().map(str::to_owned)
        }
        _ => None,
    }
}

/// The literal text a non-literal specifier starts with, if any.
fn static_prefix(source: &Expression) -> Option<String> {
    let prefix = match source {
        Expression::TemplateLiteral(t) => t.quasis.first()?.value.cooked?.as_str()?,
        Expression::BinaryExpression(b) if b.operator == BinaryOperator::Addition => {
            match &b.left {
                Expression::StringLiteral(s) => s.value.as_str()?,
                _ => return None,
            }
        }
        _ => return None,
    };
    (!prefix.is_empty()).then(|| prefix.to_owned())
}

impl<'a> Visit<'a> for Visitor<'a> {
    fn enter_node(&mut self, kind: AstKind<'a>) {
        match kind {
            AstKind::CallExpression(call)
                if call.callee.is_specific_id("require") && call.arguments.len() == 1 =>
            {
                if let Some(source) = call.arguments.first().and_then(Argument::as_expression) {
                    self.load(source, call.span, true);
                }
            }
            AstKind::ImportExpression(import) => {
                self.load(&import.source, import.span, false);
            }
            _ => {}
        }
        if let AstKind::IdentifierReference(ident) = kind {
            let name_use = self
                .file_use
                .names
                .entry(ident.name.as_str().to_owned())
                .or_default();
            let members = match self.ancestors.last() {
                Some(AstKind::StaticMemberExpression(member))
                    if member.object.span() == ident.span =>
                {
                    Some(vec![member.property.name.as_str().to_owned()])
                }
                Some(AstKind::TSQualifiedName(name)) if name.left.span() == ident.span => {
                    Some(vec![name.right.name.as_str().to_owned()])
                }
                Some(AstKind::VariableDeclarator(declarator))
                    if initializes(declarator, ident.span) =>
                {
                    destructured(&declarator.id)
                }
                _ => None,
            };
            match (members, self.ancestors.last()) {
                (Some(members), _) => name_use.members.extend(members),
                (None, Some(AstKind::ExportSpecifier(_))) => name_use.re_exported = true,
                (None, _) => name_use.escapes = true,
            }
        }
        self.ancestors.push(kind);
    }

    fn leave_node(&mut self, _kind: AstKind<'a>) {
        self.ancestors.pop();
    }
}
