//! Reading package.json `scripts` as shell commands, to find the binaries they run.

/// Runner flags that take the next word as their value.
const RUNNER_FLAGS_WITH_VALUES: &[&str] = &["-p", "--package", "-c", "--call"];
/// Runners whose next word, after any flags, is the binary they run.
const RUNNERS: &[&str] = &["npx", "bunx"];
/// Package managers whose `exec`-like subcommands run a binary.
const PACKAGE_MANAGERS: &[&str] = &["npm", "pnpm", "yarn", "bun"];
const EXEC_SUBCOMMANDS: &[&str] = &["exec", "dlx", "x"];
/// Binaries that run the command that follows their own `NAME=value` arguments.
const WRAPPERS: &[&str] = &["cross-env", "env"];

/// The binaries a script runs, in order. Paths to files are skipped, except binaries run
/// through `node_modules/.bin`.
pub fn binaries(script: &str) -> Vec<String> {
    commands(script)
        .iter()
        .flat_map(|words| command_binaries(words))
        .collect()
}

/// Splits a script into the words of each command, at `&&`, `||`, `;`, `|`, `&` and line
/// breaks outside quotes. Quotes are removed from words, and subshell parentheses dropped.
/// A `&` in a redirect, as in `2>&1` or `&>`, stays part of its word.
fn commands(script: &str) -> Vec<Vec<String>> {
    let mut commands = Vec::new();
    let mut words = Vec::new();
    let mut word = String::new();
    let mut quote = None;
    let mut chars = script.chars().peekable();
    while let Some(c) = chars.next() {
        match (quote, c) {
            (Some(q), c) if c == q => quote = None,
            (Some(_), c) => word.push(c),
            (None, '"' | '\'') => quote = Some(c),
            (None, '&') if word.ends_with('>') || chars.peek() == Some(&'>') => word.push(c),
            (None, ';' | '|' | '&' | '\n' | '(' | ')') => {
                end_word(&mut word, &mut words);
                if c != '(' && c != ')' && !words.is_empty() {
                    commands.push(std::mem::take(&mut words));
                }
            }
            (None, c) if c.is_whitespace() => end_word(&mut word, &mut words),
            (None, c) => word.push(c),
        }
    }
    end_word(&mut word, &mut words);
    if !words.is_empty() {
        commands.push(words);
    }
    commands
}

/// Adds the word being read, if any, to the command's words.
fn end_word(word: &mut String, words: &mut Vec<String>) {
    if !word.is_empty() {
        words.push(std::mem::take(word));
    }
}

/// The binaries one command runs: its first word after any `NAME=value` prefixes, looking
/// through runners and wrappers to the binary they run.
fn command_binaries(words: &[String]) -> Vec<String> {
    let mut binaries = Vec::new();
    let mut rest = words;
    loop {
        rest = skip_while(rest, is_assignment);
        let Some((first, tail)) = rest.split_first() else {
            break;
        };
        if RUNNERS.contains(&first.as_str()) {
            rest = skip_runner_flags(tail);
            continue;
        }
        if PACKAGE_MANAGERS.contains(&first.as_str())
            && tail
                .first()
                .is_some_and(|sub| EXEC_SUBCOMMANDS.contains(&sub.as_str()))
        {
            rest = skip_runner_flags(&tail[1..]);
            continue;
        }
        binaries.extend(binary_name(first));
        if !WRAPPERS.contains(&first.as_str()) {
            break;
        }
        rest = tail;
    }
    binaries
}

/// The words after a runner's flags and their values, starting at the binary it runs.
fn skip_runner_flags(mut words: &[String]) -> &[String] {
    while let Some((flag, tail)) = words.split_first() {
        if !flag.starts_with('-') {
            break;
        }
        words = if RUNNER_FLAGS_WITH_VALUES.contains(&flag.as_str()) {
            tail.get(1..).unwrap_or_default()
        } else {
            tail
        };
    }
    words
}

/// The words from the first one that `skip` doesn't match.
fn skip_while(words: &[String], skip: impl Fn(&str) -> bool) -> &[String] {
    let start = words
        .iter()
        .position(|word| !skip(word))
        .unwrap_or(words.len());
    &words[start..]
}

/// Whether a word is an env var prefix such as `NODE_ENV=production`.
fn is_assignment(word: &str) -> bool {
    word.split_once('=').is_some_and(|(name, _)| {
        !name.is_empty()
            && !name.starts_with(|c: char| c.is_ascii_digit())
            && name.chars().all(|c| c.is_ascii_alphanumeric() || c == '_')
    })
}

/// The binary a command's first word names, if it names one rather than a file.
fn binary_name(word: &str) -> Option<String> {
    match word.rsplit_once('/') {
        Some((dir, name)) if dir.ends_with("node_modules/.bin") => Some(name.to_owned()),
        Some(_) => None,
        None => Some(word.to_owned()),
    }
}
