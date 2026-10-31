import { DigSite } from "./components/dig-site";
import { InstallCommand } from "./components/install-command";
import { ThemeSwitch } from "./components/theme-switch";
import { exits, monorepo } from "./fixtures";

const REPO = "https://github.com/salmansheri/exhume";

const roadmap = [
  { item: "Unused Exports, Dependencies and env vars", when: "v1" },
  { item: "Confidence with coded Reasons", when: "v1" },
  { item: "Baselines for repos with existing dead code", when: "v1" },
  { item: <><code>--json</code> output and <code>--fail-on</code></>, key: "json", when: "v1" },
  { item: "Verified fixer that runs your build and tests", when: "later" },
  { item: "Tombstone Report pull requests", when: "later" },
];

export default function Home() {
  return (
    <>
      <div className="wrap">
        <nav className="nav" aria-label="Main">
          <a className="wordmark" href="#">
            exhume
          </a>
          <div className="nav-links">
            <a href="#how">How it reads a workspace</a>
            <a href="#ci">CI</a>
            <a href="#roadmap">Roadmap</a>
            <a className="nav-gh" href={REPO}>
              GitHub
            </a>
            <ThemeSwitch />
          </div>
        </nav>

        <main>
          <header className="surface">
            <div>
              <h1 className="display">
                Dig up the files <em>nothing imports.</em>
              </h1>
              <p className="lede">
                exhume walks your JS/TS workspace from its entry points and reports every file it never reaches.
                Across every package, in one pass.
              </p>
            </div>
            <InstallCommand note="npm and crates.io packages arrive with the first release." />
          </header>

          <DigSite {...monorepo} />
        </main>
      </div>

      <section className="layer" id="how" aria-labelledby="how-title">
        <div className="wrap">
          <h2 className="headline" id="how-title">
            It follows imports. Whatever it never reaches is dead.
          </h2>
          <p className="sub">
            No guessing from file names or timestamps. exhume builds the import graph the way your bundler would, then
            reports the files left outside it.
          </p>
          <div className="walk">
            <div>
              <h3 className="title">Start at the entry points</h3>
              <p>
                Each Workspace Package&apos;s manifest names where its code begins: <code>main</code>,{" "}
                <code>exports</code>, <code>bin</code>.
              </p>
            </div>
            <div>
              <h3 className="title">Follow every import</h3>
              <p>
                Relative paths, imports between Workspace Packages, and tsconfig <code>paths</code> aliases all resolve
                to real files.
              </p>
            </div>
            <div>
              <h3 className="title">Report what&apos;s left</h3>
              <p>Files no path reaches become Unused Files, grouped by package with the exact location.</p>
            </div>
          </div>
          <ul className="chips" aria-label="Supported workspaces">
            <li>single package</li>
            <li>pnpm workspaces</li>
            <li>npm workspaces</li>
            <li>yarn workspaces</li>
            <li>bun workspaces</li>
            <li>tsconfig paths</li>
          </ul>
        </div>
      </section>

      <section className="layer" id="ci" aria-labelledby="ci-title">
        <div className="wrap">
          <h2 className="headline" id="ci-title">
            Three exit codes. Nothing to parse.
          </h2>
          <p className="sub">
            exhume runs unattended. The exit code alone tells your pipeline whether to pass, fail, or flag a broken
            setup. Every line below is real output from exhume&apos;s test fixtures.
          </p>
          <div className="exits">
            {exits.map((exit) => (
              <div className="exit" key={exit.code}>
                <div className="exit-code" data-findings={exit.code === 1 ? "" : undefined} aria-label={`Exit code ${exit.code}`}>
                  {exit.code}
                </div>
                <div>
                  <h3 className="title">{exit.title}</h3>
                  <p>{exit.meaning}</p>
                </div>
                <pre className="output" data-wrap={exit.wrap ? "" : undefined} tabIndex={0} aria-label={`Output of ${exit.command}`}>
                  <span className="cmd">$ {exit.command}</span>
                  {"\n"}
                  {exit.output}
                </pre>
              </div>
            ))}
          </div>
          <div className="ci">
            <pre tabIndex={0} aria-label="GitHub Actions steps">
              <span className="cmd"># .github/workflows/exhume.yml</span>
              {`
- name: Install exhume
  run: >-
    cargo install --git
    ${REPO}
    exhume-cli

- name: Find dead files
  run: exhume .`}
            </pre>
            <div>
              <p>The step fails on exit 1 and the log shows exactly which files to delete.</p>
              <p>exhume never modifies your workspace. It reports, you decide.</p>
            </div>
          </div>
        </div>
      </section>

      <section className="layer" id="roadmap" aria-labelledby="roadmap-title">
        <div className="wrap">
          <h2 className="headline" id="roadmap-title">
            On the roadmap, not shipped
          </h2>
          <p className="sub">
            Unused Files is what works today. The rest of v1 is specified in the open and tracked issue by issue.
          </p>
          <div className="road">
            <ul>
              {roadmap.map((entry) => (
                <li key={entry.key ?? String(entry.item)}>
                  <strong>{entry.item}</strong>
                  <span>{entry.when}</span>
                </li>
              ))}
            </ul>
          </div>
        </div>
      </section>

      <section className="close" aria-labelledby="close-title">
        <div className="wrap">
          <div className="close-grid">
            <h2 className="headline" id="close-title">
              Run it on your monorepo.
            </h2>
            <InstallCommand
              note={
                <>
                  Then run <code>exhume</code> at the workspace root.
                </>
              }
            />
          </div>
          <footer className="footer">
            <span>exhume · MIT OR Apache-2.0</span>
            <a href={REPO}>github.com/salmansheri/exhume</a>
          </footer>
        </div>
      </section>
    </>
  );
}
