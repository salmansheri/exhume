---
version: 1
slug: "exhume-landing-app-page-tsx"
primary_target: "exhume-landing/app/page.tsx"
related_targets: []
---

# Surface: exhume landing page

Scope: the single landing page in exhume-landing/app (route `/`). Visitor mode: Persuade.

Audience and job: monorepo maintainers deciding whether exhume is believable enough to put in CI. Primary action: copy the working install command (`cargo install --git https://github.com/salmansheri/exhume exhume-cli`). Proof: exhume's real, unedited output on its own test fixtures (pnpm-monorepo, single-package, clean-package, yarn-pnp for exit 2). Light and dark themes, both full renditions; follows OS with a remembered manual switch.

Constraints: nothing unshipped presented as shipped; no npm/crates commands until published; no `--json`/`--fail-on` output; no invented numbers; never "graveyard"; red line: no toy/app-store feel.

Unresolved: logo (wordmark set in type for now); deploy target.

## Direction contract

THESIS: The workspace is one molded ground and exhume lifts the dead files out of it. Refuses the category default of a dark terminal hero beside a headline with a feature-card grid below.

OWN-WORLD: The Molded Ground (DESIGN.md). One clay per theme (Slate Clay / Night Clay), every surface pressed or raised from it with the four-shadow vocabulary, Verdigris as the only accent, Schibsted Grotesk 800 display against JetBrains Mono for real output only.

STORY: The visitor sees their own kind of repo as strata, watches the four unused files rise in report order with `exit 1`, learns it just follows imports from entry points, sees the three exit codes with real output, reads what is roadmap, and copies the install command.

FIRST VIEWPORT: Wordmark left, links and the pressed theme switch right. Two-line display headline "Dig up the files nothing imports." bottom-aligned left, raised install slab with the pressed command well and Copy key right. Below, the pressed-deep ground spans the column showing the pnpm-monorepo strata, its top edge inside the first viewport at 1280 to 1600 wide.

FORM: Dig site, position 4 on the grounded list, seed key 6d2f31c3. Signature interaction: findings rise one by one from pressed to raised (700ms, 420ms stagger), then the summary and exit keycap appear; Scan again replays; reduced motion and no-JS show the final state.

FINISH: unreviewed and undocumented is unfinished; this build ends with the finish review, the verdict, DESIGN.md, and every shipping raster carrying its provenance
