# Exhume

Exhume finds code, dependencies and configuration in a monorepo that nothing uses, and reports them as findings that a person or the fixer can act on.

## Repository structure

**Ecosystem**:
A family of languages and tooling that shares one manifest format, module system and package registry, such as JS (package.json and npm). v1 only understands the JS ecosystem.
_Avoid_: Language, platform, stack

**Workspace**:
The whole monorepo being scanned, made up of one or more workspace packages. A repo that isn't a monorepo is a workspace with one package.
_Avoid_: Project, repo (when you mean the scanned unit)

**Workspace Package**:
A directory inside the workspace that has its own manifest and is listed by the workspace configuration.
_Avoid_: Module, project, sub-repo

**Dependency**:
An external package that a workspace package declares in its manifest.
_Avoid_: Package (on its own), library, module

**Public API**:
The exports reachable from the published entry points of a workspace package that isn't marked private. Code outside the workspace may use them, so by default they count as used.
_Avoid_: Library exports, external surface

## Reachability

**Entry Point**:
A file that exhume assumes is used without needing any importer, such as a manifest entry, a framework convention file, a test or a config file. Reachability starts from these files.
_Avoid_: Root, main file, seed

**Entry Provider**:
A source of entry points for one tool or framework, such as Next, Vite or a test runner. It is only active when its tool is present in a workspace package.
_Avoid_: Plugin, adapter (the Routes adapter is a separate concept)

**Reachable**:
A file or export is reachable when a chain of imports leads to it from some entry point.
_Avoid_: Used (on its own), live, referenced

## Findings

**Finding**:
One thing that exhume believes is unused. It has a kind, a location, a confidence and the reasons behind that confidence.
_Avoid_: Issue, warning, result, violation

**Finding ID**:
A stable identity for a finding. It is built from the kind, workspace package, path and name, and it stays the same when lines move. Ignore rules and baselines refer to it.
_Avoid_: Fingerprint, hash, key

**Unused File**:
A finding for a source file that is not reachable from any entry point. It includes the file's exports, which are not reported separately.
_Avoid_: Orphan file, dead file

**Unused Export**:
A finding for an export of a reachable file that nothing imports, excluding public API.
_Avoid_: Dead export

**Unused Dependency**:
A finding for a declared dependency that no source file, config file or script uses.
_Avoid_: Unused package, extraneous dependency

**Unused Route**:
An experimental finding for a framework route that no link, navigation call or path literal in the workspace points to. It is always Low.
_Avoid_: Dead page, orphan route

**Baseline**:
A recorded set of Finding IDs that a team has accepted for now. Only findings outside the baseline fail CI.
_Avoid_: Allowlist, snapshot, suppressions

**Confidence**:
How safe a finding is to act on: High, Medium or Low. The fixer only acts on High.
_Avoid_: Score, severity, certainty

**Reason**:
A coded explanation of why a finding's confidence is lower than High, for example a namespace import that escapes, or an unparsed file next to it.
_Avoid_: Note, message, hint

**Unused Env Var**:
A finding for an env var that is documented somewhere but never read by any source file.
_Avoid_: Dead variable, stale env

**Undocumented Env Var**:
A finding for an env var that source code reads but no env documentation lists.
_Avoid_: Missing env, unknown variable

**Env Documentation**:
A place that declares which env vars a workspace expects without holding their real values, such as `.env.example`, Dockerfiles, compose files, CI workflows and `turbo.json`.
_Avoid_: Env config, env file (real `.env` files are not env documentation)

## Fixing

**Fixer**:
The part of exhume that turns High findings into code changes and keeps only the changes that pass verification.
_Avoid_: Autofix, cleaner, remover

**Batch**:
All High findings of one kind that the fixer applies and verifies together.
_Avoid_: Chunk, group, changeset

**Verification**:
Running the user's build and test commands against a batch. A batch that fails verification is bisected, and the findings that break it are dropped.
_Avoid_: Check, validation, CI run

**Tombstone Report**:
The pull request description listing each removed thing, together with who last changed it and when.
_Avoid_: Changelog, removal summary
