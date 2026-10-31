// Captured from exhume-cli/tests/fixtures. Findings, summaries, exit codes and command output are verbatim exhume output;
// the reachable (non-finding) files in each stratum are the fixture's real file tree, not CLI output.

export type Stratum = {
  name: string;
  path: string;
  files: { label: string; finding?: boolean }[];
};

export const monorepo = {
  command: "exhume tests/fixtures/pnpm-monorepo",
  strata: [
    {
      name: "web",
      path: "apps/web",
      files: [
        { label: "src/index.ts" },
        { label: "src/lib/aliased.ts" },
        { label: "apps/web/src/dead.ts:1:1", finding: true },
      ],
    },
    {
      name: "@acme/ui",
      path: "packages/ui",
      files: [
        { label: "src/index.ts" },
        { label: "packages/ui/src/internal.ts:1:1", finding: true },
        { label: "packages/ui/src/unused.ts:1:1", finding: true },
      ],
    },
    {
      name: "@acme/utils",
      path: "packages/utils",
      files: [
        { label: "index.ts" },
        { label: "src/format.ts" },
        { label: "src/extra.ts" },
        { label: "packages/utils/src/stale.ts:1:1", finding: true },
      ],
    },
  ] satisfies Stratum[],
  summary: "4 findings (4 high, 0 medium, 0 low)",
  exit: 1,
};

export const exits = [
  {
    code: 0,
    title: "Clean",
    meaning: "Every file is reachable.",
    command: "exhume tests/fixtures/clean-package",
    output: "0 findings (0 high, 0 medium, 0 low)",
  },
  {
    code: 1,
    title: "Findings",
    meaning: "Dead files exist. Fail the check.",
    command: "exhume tests/fixtures/single-package",
    output: `single-package (.)
  Unused Files (4)
    src/Widget.tsx:1:1                               high
    src/dead-helper.ts:1:1                           high
    src/dead.ts:1:1                                  high
    src/legacy.cjs:1:1                               high

4 findings (4 high, 0 medium, 0 low)`,
  },
  {
    code: 2,
    wrap: true,
    title: "Tool error",
    meaning: "exhume couldn't scan, and says why.",
    command: "exhume tests/fixtures/yarn-pnp",
    output:
      "exhume: Yarn Plug'n'Play (PnP) is not supported; set `nodeLinker: node-modules` in .yarnrc.yml and reinstall",
  },
];
