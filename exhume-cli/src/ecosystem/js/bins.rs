//! What exhume knows about binaries without reading the manifests of installed Dependencies.

/// Well-known binaries and the Dependencies that install them, for when they aren't installed.
const OWNERS: &[(&str, &[&str])] = &[
    ("astro", &["astro"]),
    ("babel", &["@babel/cli"]),
    ("biome", &["@biomejs/biome"]),
    ("c8", &["c8"]),
    ("changeset", &["@changesets/cli"]),
    ("concurrently", &["concurrently"]),
    ("cross-env", &["cross-env"]),
    ("cypress", &["cypress"]),
    ("drizzle-kit", &["drizzle-kit"]),
    ("esbuild", &["esbuild"]),
    ("eslint", &["eslint"]),
    ("husky", &["husky"]),
    ("jest", &["jest"]),
    ("lerna", &["lerna"]),
    ("lint-staged", &["lint-staged"]),
    ("mocha", &["mocha"]),
    ("next", &["next"]),
    ("nodemon", &["nodemon"]),
    ("npm-run-all", &["npm-run-all", "npm-run-all2"]),
    ("nuxi", &["nuxi", "nuxt"]),
    ("nuxt", &["nuxt"]),
    ("nx", &["nx"]),
    ("nyc", &["nyc"]),
    ("oxlint", &["oxlint"]),
    ("playwright", &["@playwright/test", "playwright"]),
    ("postcss", &["postcss-cli"]),
    ("prettier", &["prettier"]),
    ("prisma", &["prisma"]),
    ("remix", &["@remix-run/dev"]),
    ("rimraf", &["rimraf"]),
    ("rollup", &["rollup"]),
    ("run-p", &["npm-run-all", "npm-run-all2"]),
    ("run-s", &["npm-run-all", "npm-run-all2"]),
    ("storybook", &["storybook"]),
    ("stylelint", &["stylelint"]),
    ("svelte-check", &["svelte-check"]),
    ("svelte-kit", &["@sveltejs/kit"]),
    ("tailwindcss", &["tailwindcss", "@tailwindcss/cli"]),
    ("ts-node", &["ts-node"]),
    ("tsc", &["typescript"]),
    ("tsserver", &["typescript"]),
    ("tsup", &["tsup"]),
    ("tsx", &["tsx"]),
    ("turbo", &["turbo"]),
    ("vite", &["vite"]),
    ("vitest", &["vitest"]),
    ("vue-tsc", &["vue-tsc"]),
    ("webpack", &["webpack"]),
    ("wrangler", &["wrangler"]),
];

/// Commands a script can run that no Dependency installs: shell builtins, common system
/// tools and the JS runtimes and package managers themselves.
const NOT_DEPENDENCIES: &[&str] = &[
    ".", "[", "awk", "bash", "bun", "cat", "cd", "chmod", "command", "corepack", "cp", "curl",
    "cut", "deno", "docker", "echo", "env", "eval", "exec", "exit", "export", "false", "find",
    "git", "grep", "head", "kill", "ln", "ls", "make", "mkdir", "mv", "node", "npm", "pnpm",
    "printf", "pwd", "read", "rm", "sed", "set", "sh", "sleep", "sort", "source", "tail", "tar",
    "tee", "test", "touch", "tr", "true", "type", "wait", "wc", "which", "xargs", "yarn",
];

/// The Dependencies the table says install `binary`, or `None` if it doesn't know the binary.
pub fn guessed_owners(binary: &str) -> Option<&'static [&'static str]> {
    OWNERS
        .binary_search_by_key(&binary, |(name, _)| name)
        .ok()
        .map(|index| OWNERS[index].1)
}

/// Whether `command` is known not to be any Dependency's binary.
pub fn is_not_a_dependency(command: &str) -> bool {
    NOT_DEPENDENCIES.contains(&command)
}
