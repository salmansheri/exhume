# Product

<!-- impeccable:product-schema 1 -->

## Platform

web

## Users

The primary users are monorepo maintainers: senior and staff engineers and tech leads who own a JS/TS monorepo. Their job is to remove dead code (Unused Files, Unused Exports, Unused Dependencies, unused and undocumented env vars) without breaking anything. They mostly run exhume in CI, where it has to be fast and believable enough to gate merges.

## Product Purpose

Exhume is a Rust CLI. It scans a JS-ecosystem workspace and reports what nothing uses. Each Finding carries a Confidence (High, Medium or Low) and coded Reasons that explain any downgrade. A Baseline lets a repo that already has dead code adopt exhume without failing CI on day one. Exhume succeeds when maintainers trust the report enough to act on it and leave it running in CI.

The site (`exhume-landing/`) introduces the CLI to those maintainers and gets them to install it.

## Positioning

The claim is trustworthy findings, fast. Exhume reports with confidence levels and reasons, and when it is unsure it says nothing rather than guessing. Exhume's own mechanism is the pitch, and the site does not name competing tools.

On the roadmap, not shipped: a verified fixer that applies High findings in a scratch worktree, runs the user's build and tests, and bisects failures; and GitHub PRs with a Tombstone Report saying what was removed, who last touched it and when.

## Operating Context

- Exhume runs in a terminal and in CI, for example GitHub Actions. Its output is grouped text or `--json`.
- Exit codes: `0` when clean, `1` when there are findings at or above `--fail-on` (default `medium`), `2` on a tool error.
- Configuration is a single `exhume.toml` at the workspace root. In code, the JSDoc tag `@public` and the comment `// exhume-ignore-next-line` are honoured.
- Exhume is installed from npm (`exhume`, with per-platform binaries), crates.io (`cargo install exhume-cli`) or GitHub Releases.
- The repository is `github.com/salmansheri/exhume`, licensed MIT OR Apache-2.0.

## Capabilities and Constraints

- v1 reports only. It never modifies the workspace. Shipped capabilities so far: Unused Files in single-package workspaces and monorepos (pnpm, npm, yarn and bun workspaces, imports between Workspace Packages, tsconfig `paths`). The rest of v1 is tracked in issues #4–#23 of the spec, salmansheri/exhume#1.
- v1 covers the JS ecosystem only: `.js/.ts/.jsx/.tsx/.mjs/.cjs`, script blocks of `.vue/.svelte/.astro`, ESM in `.mdx`, and import edges in `.css/.scss` (ADR 0001). Other ecosystems are later milestones.
- Entry Providers planned for v1: plain manifest, Next, Nuxt, SvelteKit, Astro, Remix and Vite, plus tooling (Vitest, Jest, Playwright, Storybook, ESLint, Prettier, PostCSS, Tailwind, TypeScript).
- Exhume never reads real `.env` files (ADR 0002).
- Yarn Plug'n'Play is not supported.
- Terminology follows `GLOSSARY.md`, for example Finding, Unused Dependency (not "unused package"), Workspace Package, Entry Point and Baseline.
- Site stack: Next.js 16 with Tailwind CSS 4, run with bun, in `exhume-landing/`.

## Brand Commitments

- The name is **exhume**, always lowercase as a command. The product was renamed from "graveyard" because that name was taken on crates.io and npm, so "graveyard" must not be used as the product name.
- Theme: digging up what is dead in a codebase. The roadmap PR artifact is called the Tombstone Report.
- Voice: honest about what exists. Shipped features are presented as shipped, and roadmap items are labelled clearly as roadmap.

## Evidence on Hand

- The open-source repo, the spec (salmansheri/exhume#1) and the fixture test suite.
- There are no benchmarks, users, testimonials, download counts or case studies yet. Do not invent any, and do not publish speed numbers until real benchmarks exist.
- There is no logo or other brand asset yet. The site still uses the create-next-app scaffold assets.

## Product Principles

1. **Trust over coverage.** A finding nobody believes is worse than no finding. When exhume is unsure it stays quiet, except where staying quiet would hide whole Unused Files.
2. **Every downgrade explains itself.** Confidence always comes with a coded Reason.
3. **Adoptable on day one.** Baselines and thresholds let real repos with existing dead code switch exhume on in CI.
4. **Honest about the roadmap.** Never present unshipped capability as shipped.
5. **Built for CI.** Output, exit codes and speed serve unattended runs first.
