# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## What this is

**exhume** is a Rust CLI that scans a JS/TS workspace (single package or pnpm/npm/yarn/bun monorepo) and reports what nothing uses: Unused Files, Unused Exports, Unused Dependencies (env vars and routes are planned). Each Finding has a Confidence (High/Medium/Low) and coded Reasons explaining any downgrade. v1 only reports and never modifies the scanned workspace.

The repo has two independent projects:

- `exhume-cli/`: the Rust crate (lib `exhume`, binary `exhume`).
- `exhume-landing/`: the marketing site (Next.js 16, React 19, Tailwind 4, run with bun).

The directory is named `graveyard`, but the product is **exhume**. Never use "graveyard" as the product name (see `PRODUCT.md`).

## Commands

CLI (run from `exhume-cli/`):

```bash
cargo build
cargo test                                   # all tests
cargo test --test scan                       # library-level fixture tests
cargo test --test cli                        # binary exit-code tests
cargo test --test scan barrels               # one test by name filter
cargo insta review                           # accept changed insta snapshots (tests/snapshots/)
cargo run -- path/to/workspace [--include-public-api]
```

Exit codes: `0` clean, `1` findings exist, `2` tool error (e.g. Yarn Plug'n'Play, which is rejected).

Landing site (run from `exhume-landing/`): `bun run dev` (port 3000), `bun run build`, `bun run lint`. Its `AGENTS.md` warns that this Next.js version differs from training data: read `node_modules/next/dist/docs/` before writing Next code.

## Architecture (exhume-cli)

`lib.rs::scan` is the whole pipeline:

1. `JsEcosystem::load` discovers Workspace Packages (manifests, workspace globs, tsconfig paths).
2. `walk::files` collects candidate source files (respecting ignore files plus the ecosystem's always-skipped dirs).
3. `reachable_files` does a BFS from every package's Entry Points, parsing each file once into a `Module` and resolving its specifiers. Anything not reached is an Unused File.
4. `blind_spots.rs` finds places exhume can't see into (non-literal `import()`, `require()`, unresolved imports, …) and scopes them to a file, dir or package. Every finding in that scope gets `lowered_by(reasons)`, which lowers its Confidence.
5. `exports.rs` follows imports, re-exports, barrels and namespaces over the reachable set to find Unused Exports. Exports of Public API (entry points of non-private packages) count as used unless `--include-public-api` is passed. `@public` JSDoc and `exhume-ignore-next-line` are honoured.
6. `dependencies.rs` reports declared Dependencies that no source, config or script uses.
7. `report.rs` groups findings by package with stable sorting and a summary. `finding.rs` defines kinds, Confidence, ReasonCodes and the stable Finding ID (kind + package + path + name, not line).

**Ecosystem seam (ADR 0001):** `src/ecosystem/mod.rs` defines the `Ecosystem` trait and the neutral types (`Module`, `Import`, `Export`, `ReExport`, `Resolution`, `Dependency`). `src/ecosystem/js/` is the only implementation and the only place allowed to use `oxc_*` types (parser in `parse.rs`, `oxc_resolver` in `resolve.rs`, manifests/workspaces in `manifest.rs`/`workspace.rs`, dependency usage in `usage.rs`). The core must not depend on oxc types. The `oxc_*` crates are pinned with `=` versions on purpose.

**Never read real `.env` files (ADR 0002):** env vars are only checked against documentation files (`.env.example`, Dockerfiles, CI workflows, `turbo.json`, …).

## Tests

Tests are fixture-driven. Each `tests/fixtures/<name>/` is a small workspace with an `expected.json` listing every planted finding (`kind, package, path, name, line, column, confidence, reasons`). `tests/scan.rs::assert_expected` checks that the report contains exactly those findings. A new behaviour usually means a new fixture plus `expected.json` and a test in `tests/scan.rs`. Fixtures that need option-specific results use a second file, e.g. `expected-include-public-api.json`.

## Domain language

Read `GLOSSARY.md` before naming things. Use its terms (Finding, Workspace Package, Entry Point, Entry Provider, Reachable, Public API, Confidence, Reason, Baseline, Unused Dependency) and avoid the synonyms it lists. Read the relevant `docs/adr/` before changing an area, and say so explicitly if a change contradicts an ADR. `PRODUCT.md` holds the product scope and brand rules (don't present roadmap features as shipped, don't invent benchmarks). `DESIGN.md` holds the landing site's design tokens.

## Agent skills

### Issue tracker

Issues and specs are GitHub Issues in salmansheri/exhume, used through the `gh` CLI. The v1 spec is salmansheri/exhume#1. See `docs/agents/issue-tracker.md`.

### Triage labels

The default labels: needs-triage, needs-info, ready-for-agent, ready-for-human, wontfix. See `docs/agents/triage-labels.md`.

### Domain docs

Single-context: `GLOSSARY.md` and `docs/adr/` at the repo root. See `docs/agents/domain.md`.
