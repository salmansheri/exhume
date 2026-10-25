# v1 supports only the JS ecosystem, behind an Ecosystem seam

v1 analyses only the JS ecosystem: `.js/.ts/.jsx/.tsx/.mjs/.cjs`, the script blocks of `.vue/.svelte/.astro`, ESM in `.mdx`, and import edges in `.css/.scss`. All parsing, resolution, manifest reading and entry providers sit behind an `Ecosystem` trait, so that Rust, Java, C# and others can be added later as separate ecosystems.

## Considered Options

- **Polyglot from day one, using tree-sitter for everything.** Rejected. Every language needs its own resolver (Cargo modules, the Maven/Gradle classpath, C/C++ include paths decided by the build system), its own manifest model and its own entry-point rules, so each one costs about as much as all of v1. For C/C++, static reachability depends on the build configuration and macros, so High confidence would rarely be honest. Rust already reports dead code through rustc and cargo-machete.
- **JS only, with no seam.** Rejected. Adding other ecosystems is a stated goal, and adding the seam later would mean rewriting the core.

## Consequences

- `oxc` is the JS ecosystem's implementation, not part of the core. The core graph, findings, report and fixer must not depend on oxc types.
