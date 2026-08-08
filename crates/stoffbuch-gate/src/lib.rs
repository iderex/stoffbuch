//! The gate.
//!
//! Holds every check that guards the register and runs them in order, and
//! holds the accounting that says what was examined and what was not. It is a
//! library rather than a binary so that the one command a person runs and the
//! workflow that runs it are the same procedure rather than two.
//!
//! The set of parts lives here and nowhere else. A document that listed them
//! would drift against this file the first time a part is added, so a document
//! that needs to talk about the set points at the command instead.

use std::fmt::Write as _;
use std::io::{self, Write};
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::{Duration, Instant};

/// What a run of the gate is worth to a caller.
///
/// The three values are the ones `docs/decisions/error-and-failure-policy.md`
/// names. A script can act only on what it can tell apart, so the set is small
/// and each value means one thing.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Exit {
    /// Every part that had something to run ran, and none of them refused.
    Clean,
    /// A part refused. The tree is wrong and the run says which part and why.
    Refused,
    /// The gate could not judge. It could not find the workspace, could not
    /// start a part, or could not write its own report.
    CouldNotJudge,
}

impl Exit {
    /// The process exit code for this verdict.
    #[must_use]
    pub fn code(self) -> u8 {
        match self {
            Self::Clean => 0,
            Self::Refused => 1,
            Self::CouldNotJudge => 2,
        }
    }
}

/// What a part did.
#[derive(Debug, Clone, PartialEq, Eq)]
enum Outcome {
    /// It ran and refused nothing. The note, where there is one, says what the
    /// part did not reach, so a green line is not read as covering it.
    Ran(Duration, Option<String>),
    /// It did not run, and the sentence says what would make it.
    Skipped(&'static str),
    /// It ran and refused. The string is everything it wrote.
    Failed(Duration, String),
    /// It could not be started at all, so nothing about its subject is known.
    Unstartable(String),
    /// The run stopped at an earlier failure and never got here.
    NotReached,
}

/// One part of the gate, and what it did, once the run has passed it.
#[derive(Debug, Clone, PartialEq, Eq)]
struct Reported {
    name: &'static str,
    examines: &'static str,
    outcome: Outcome,
}

/// How a part is run.
enum Runs {
    /// A command, the program first and its arguments after, run with the
    /// workspace root as the working directory.
    Command(&'static [&'static str]),
    /// A check this repository owns, run in this process against the tree at
    /// the given root.
    Check(fn(&Path) -> Judged),
    /// Nothing yet. The part is declared anyway, so that a run cannot be read
    /// as covering what the part names, and the sentence says what would make
    /// it run.
    NotBuilt(&'static str),
}

/// What a check this repository owns found.
#[derive(Debug, Clone, PartialEq, Eq)]
enum Judged {
    /// Nothing refused. The note, where there is one, says what the check did
    /// not reach.
    Nothing(Option<String>),
    /// Refused. The text names every subject and why.
    Refused(String),
    /// The check could not judge its subject, so nothing about it is known.
    CouldNotJudge(String),
}

/// A part of the gate.
struct Part {
    /// What the part is called in the report.
    name: &'static str,
    /// One line saying what the part examines, in the report beside its
    /// verdict, so a reader knows what a green line covered.
    examines: &'static str,
    /// How it is run.
    runs: Runs,
}

/// Every part, in the order they run.
///
/// The order is cheapest first among the parts that read the same files, so a
/// run ends at the thing to fix rather than after the slowest part. Formatting
/// comes before linting because a reformat changes the source the linter reads.
const PARTS: &[Part] = &[
    Part {
        name: "line-endings",
        examines: "every tracked text file in the working copy, for a carriage return",
        runs: Runs::Check(no_carriage_return),
    },
    Part {
        name: "headless",
        examines: "the test code in every tracked Rust file, for a surface a test may not reach",
        runs: Runs::Check(a_test_reaches_nothing_it_may_not),
    },
    Part {
        name: "format",
        examines: "every Rust file in the workspace, against the form rustfmt writes",
        // `--color=never` is passed through to rustfmt, which colours its diff
        // whether or not anything is watching. What a part wrote is quoted
        // into a report and into an issue, and terminal escapes in a quoted
        // refusal are noise a reader has to look past.
        runs: Runs::Command(&["cargo", "fmt", "--all", "--check", "--", "--color=never"]),
    },
    Part {
        name: "lint",
        examines: "every crate and every test target, at the workspace lint levels",
        runs: Runs::Command(&[
            "cargo",
            "clippy",
            "--workspace",
            "--all-targets",
            "--locked",
        ]),
    },
    Part {
        name: "schema",
        examines: "every file under register/, against the schema for its row kind",
        runs: Runs::NotBuilt(
            "a schema check exists and there is a schema under schema/ for it to read",
        ),
    },
    Part {
        name: "tests",
        examines: "the test suite of every crate in the workspace",
        runs: Runs::Command(&["cargo", "test", "--workspace", "--locked"]),
    },
];

/// Runs every part in order, stopping at the first failure, writing the report
/// to `out` as it goes.
///
/// `root` is the workspace root, and every part runs with it as the working
/// directory, so the verdict is about the tree rather than about where the
/// command was typed.
///
/// # Errors
///
/// Returns the write error if the report cannot be written. A caller that
/// cannot report what it examined has not judged the tree, whatever the parts
/// said, so this is a `CouldNotJudge` and never a verdict.
pub fn run(root: &Path, out: &mut dyn Write) -> io::Result<Exit> {
    let mut results = Vec::with_capacity(PARTS.len());
    let mut stopped = false;

    for part in PARTS {
        let outcome = if stopped {
            Outcome::NotReached
        } else {
            let outcome = execute(part, root);
            stopped = matches!(outcome, Outcome::Failed(..) | Outcome::Unstartable(_));
            outcome
        };
        let result = Reported {
            name: part.name,
            examines: part.examines,
            outcome,
        };
        out.write_all(line(&result, width()).as_bytes())?;
        out.flush()?;
        results.push(result);
    }

    out.write_all(tail(&results).as_bytes())?;
    out.flush()?;
    Ok(verdict(&results))
}

/// Runs one part and says what it did.
fn execute(part: &Part, root: &Path) -> Outcome {
    let argv = match part.runs {
        Runs::Command(argv) => argv,
        Runs::NotBuilt(would_run_when) => return Outcome::Skipped(would_run_when),
        Runs::Check(check) => {
            let started = Instant::now();
            let judged = check(root);
            let took = started.elapsed();
            return match judged {
                Judged::Nothing(note) => Outcome::Ran(took, note),
                Judged::Refused(why) => Outcome::Failed(took, why),
                Judged::CouldNotJudge(why) => Outcome::Unstartable(why),
            };
        }
    };
    let (program, arguments) = argv
        .split_first()
        .expect("a part's command names a program");

    let started = Instant::now();
    let finished = Command::new(program)
        .args(arguments)
        .current_dir(root)
        .output();
    let took = started.elapsed();

    match finished {
        Err(why) => Outcome::Unstartable(format!("{}: {why}", argv.join(" "))),
        Ok(output) if output.status.success() => Outcome::Ran(took, None),
        Ok(output) => {
            let mut said = String::from_utf8_lossy(&output.stdout).into_owned();
            said.push_str(&String::from_utf8_lossy(&output.stderr));
            Outcome::Failed(took, said)
        }
    }
}

/// Refuses a tracked text file whose working copy carries a carriage return.
///
/// The stored format requires line feed endings, and a row points at its
/// tabulated block by a digest of that block's bytes, so a carriage return is
/// not a cosmetic difference: a digest computed over a working copy that
/// carries one does not match a digest computed anywhere else, and the failure
/// runs in both directions.
///
/// `.gitattributes` converts, and converting is not refusing. A file that was
/// fixed on one machine and a file that is right everywhere look the same
/// afterwards, and only the second is a guard. This is the guard: it reads the
/// working copy rather than the blob, so it goes red both when a carriage
/// return is in the tree and when a clone's checkout put one in a file the
/// declaration failed to reach.
fn no_carriage_return(root: &Path) -> Judged {
    let tracked = match tracked(root) {
        Ok(tracked) => tracked,
        Err(why) => return Judged::CouldNotJudge(why),
    };

    let mut carrying = Vec::new();
    let mut read = 0_usize;
    let mut binary = 0_usize;
    let mut unread = 0_usize;

    for name in tracked {
        let Ok(bytes) = std::fs::read(root.join(&name)) else {
            // Tracked and not in the working copy: staged for deletion, or a
            // path this filesystem cannot open. Counted rather than passed
            // over, because a file nothing read is not a file nothing refused.
            unread += 1;
            continue;
        };
        if is_binary(&bytes) {
            binary += 1;
        } else {
            read += 1;
            if carries_a_carriage_return(&bytes) {
                carrying.push(name);
            }
        }
    }

    if !carrying.is_empty() {
        let mut why = format!(
            "{} tracked text file(s) carry a carriage return, and a digest over \
             these bytes is wrong for every other reader:\n\n",
            carrying.len()
        );
        for name in &carrying {
            let _ = writeln!(why, "  {name}");
        }
        let _ = writeln!(
            why,
            "\nWhere the tree holds line feeds and the working copy does not, the \
             checkout\npredates the declaration in .gitattributes and git writes the \
             file again if\nit is removed and checked out. Where the tree holds the \
             carriage return, the\nrepair is a commit. `git ls-files --eol <path>` says \
             which of the two it is."
        );
        return Judged::Refused(why);
    }

    let mut note = format!("{read} text file(s) read");
    if binary > 0 {
        let _ = write!(note, ", {binary} skipped as binary and not judged");
    }
    if unread > 0 {
        let _ = write!(note, ", {unread} tracked but not in the working copy");
    }
    Judged::Nothing(Some(note))
}

/// Every path git tracks at `root`, or the reason the list could not be had.
///
/// git is asked rather than the directory walked, because what is tracked is
/// git's answer and a walk would judge whatever a working directory happens to
/// be holding.
fn tracked(root: &Path) -> Result<Vec<String>, String> {
    let listed = Command::new("git")
        .args(["ls-files", "-z"])
        .current_dir(root)
        .output();
    let listed = match listed {
        Ok(output) if output.status.success() => output.stdout,
        Ok(output) => {
            return Err(format!(
                "git ls-files -z said: {}",
                String::from_utf8_lossy(&output.stderr).trim_end()
            ));
        }
        Err(why) => return Err(format!("git ls-files -z: {why}")),
    };
    Ok(listed
        .split(|byte| *byte == 0)
        .filter(|name| !name.is_empty())
        .map(|name| String::from_utf8_lossy(name).into_owned())
        .collect())
}

/// Whether a file carries a carriage return anywhere in it.
fn carries_a_carriage_return(bytes: &[u8]) -> bool {
    bytes.contains(&b'\r')
}

/// A surface a test may not reach, with the rule it comes from.
///
/// The spelling is matched as plain text, so that a refusal can print the rule
/// rather than a pattern, and so that what the check refuses is a list somebody
/// can read rather than an expression somebody has to decode.
struct Forbidden {
    /// The text that names the surface.
    spelling: &'static str,
    /// The rule, in words a contributor can act on without opening this file.
    rule: &'static str,
}

/// A test may reach none of these.
///
/// Every entry is one of the things
/// `docs/decisions/headless-and-unprivileged.md` writes as a thing a test may
/// not do, and this list is the part of that record a machine can refuse. What
/// it cannot refuse is written in the record beside the rule rather than here.
const FORBIDDEN: &[Forbidden] = &[
    Forbidden {
        spelling: "DISPLAY",
        rule: "a test may not require a display server or read a display \
               environment variable to decide what to do",
    },
    Forbidden {
        spelling: "TcpStream",
        rule: "a test may not require a socket to connect",
    },
    Forbidden {
        spelling: "TcpListener",
        rule: "a test may not listen on a socket",
    },
    Forbidden {
        spelling: "UdpSocket",
        rule: "a test may not require a socket",
    },
    Forbidden {
        spelling: "to_socket_addrs",
        rule: "a test may not require a name to resolve",
    },
    Forbidden {
        spelling: "sudo",
        rule: "a test may not require administrative rights or a prompt that asks for them",
    },
    Forbidden {
        spelling: "runas",
        rule: "a test may not require elevation",
    },
    Forbidden {
        spelling: "schtasks",
        rule: "a test may not register a scheduled task or anything that outlives it",
    },
    Forbidden {
        spelling: "sc.exe",
        rule: "a test may not register or start a service",
    },
    Forbidden {
        spelling: "dev-certs",
        rule: "a test may not run anything that raises a trust or consent prompt",
    },
];

/// Refuses a test that reaches a surface the suite may not depend on.
///
/// What it reads is test code in tracked Rust files: everything from the first
/// `#[cfg(test)]` in a file to the end of it, which is where this repository
/// puts a test module, and the whole of any file under a `tests/` directory.
/// Production code is not judged, because the rule is about the suite: the
/// command line legitimately reaches a terminal and the gate legitimately
/// starts a process.
///
/// The limit is the same one the record states. This reads text, so it catches
/// the ordinary mistake written in the ordinary spelling, and it cannot see a
/// library that opens a window three calls down, a path assembled at run time,
/// or a name resolved inside a dependency. The demonstration on a machine with
/// none of those things is the other half and neither substitutes for the
/// other.
fn a_test_reaches_nothing_it_may_not(root: &Path) -> Judged {
    let tracked = match tracked(root) {
        Ok(tracked) => tracked,
        Err(why) => return Judged::CouldNotJudge(why),
    };

    let mut offences = Vec::new();
    let mut read = 0_usize;

    let rust = tracked.iter().filter(|name| {
        Path::new(name)
            .extension()
            .is_some_and(|extension| extension.eq_ignore_ascii_case("rs"))
    });
    for name in rust {
        let Ok(text) = std::fs::read_to_string(root.join(name)) else {
            continue;
        };
        read += 1;
        for (line, spelling, rule) in reached(&text, is_a_test_file(name)) {
            offences.push(format!("  {name}:{line}  {spelling}\n      {rule}"));
        }
    }

    if offences.is_empty() {
        return Judged::Nothing(Some(format!(
            "{read} tracked Rust file(s) read, test code only"
        )));
    }
    let mut why = format!("{} thing(s) a test may not reach:\n\n", offences.len());
    for offence in &offences {
        let _ = writeln!(why, "{offence}");
    }
    let _ = writeln!(
        why,
        "A test that genuinely needs one of these is not a test in this suite. \
         It moves\nto the harness for what needs a network or a long run, and \
         the move is the repair\nrather than an exception."
    );
    Judged::Refused(why)
}

/// Whether every line of a file is test code.
fn is_a_test_file(name: &str) -> bool {
    name.contains("/tests/") || name.starts_with("tests/")
}

/// Every forbidden surface the test code in `text` reaches, with its line.
///
/// Where `whole` is false, the test code is taken to start at the first
/// `#[cfg(test)]` and run to the end of the file. That is a convention rather
/// than a parse, and it is the convention this repository follows: a test
/// module with production code after it is outside what this reads, which is
/// the limit of taking a line at a time instead of carrying a parser.
fn reached(text: &str, whole: bool) -> Vec<(usize, &'static str, &'static str)> {
    let mut reaching = Vec::new();
    let mut inside = whole;
    for (index, line) in text.lines().enumerate() {
        if !inside {
            inside = line.trim_start().starts_with("#[cfg(test)]");
            continue;
        }
        for forbidden in FORBIDDEN {
            if line.contains(forbidden.spelling) {
                reaching.push((index + 1, forbidden.spelling, forbidden.rule));
            }
        }
    }
    reaching
}

/// Whether the check treats a file as binary and does not judge it.
///
/// A zero byte near the start is git's own test, and using the same one keeps
/// this check's subject the same as the subject `.gitattributes` reaches. It is
/// a heuristic in both places: a text file holding a zero byte is skipped here
/// and left unconverted there, which is why a skipped file is counted in the
/// report rather than passed over quietly.
fn is_binary(bytes: &[u8]) -> bool {
    bytes.iter().take(8000).any(|byte| *byte == 0)
}

/// The width the part names are padded to, so the verdicts line up.
fn width() -> usize {
    PARTS.iter().map(|part| part.name.len()).max().unwrap_or(0)
}

/// The report line for one part, and for a failed one everything it wrote.
///
/// Nothing here varies between two runs over the same tree except the seconds,
/// which is what makes a second run comparable to a first.
fn line(result: &Reported, width: usize) -> String {
    let mut line = String::new();
    let name = result.name;
    match &result.outcome {
        Outcome::Ran(took, note) => {
            let _ = writeln!(
                line,
                "{name:width$}  ran      {:>7}  {}",
                secs(*took),
                result.examines
            );
            if let Some(note) = note {
                let _ = writeln!(line, "{:width$}  {note}", "");
            }
        }
        Outcome::Skipped(would_run_when) => {
            let _ = writeln!(line, "{name:width$}  skipped          {}", result.examines);
            let _ = writeln!(line, "{:width$}  runs when {would_run_when}", "");
        }
        Outcome::Failed(took, said) => {
            let _ = writeln!(
                line,
                "{name:width$}  refused  {:>7}  {}",
                secs(*took),
                result.examines
            );
            let _ = writeln!(line, "\n{name} refused. Everything it wrote:\n");
            let _ = writeln!(line, "{}", said.trim_end());
        }
        Outcome::Unstartable(why) => {
            let _ = writeln!(line, "{name:width$}  unknown          {}", result.examines);
            let _ = writeln!(line, "\n{name} could not be started, so nothing about that");
            let _ = writeln!(line, "is known either way:\n\n{why}\n");
        }
        Outcome::NotReached => {
            let _ = writeln!(line, "{name:width$}  not reached      {}", result.examines);
        }
    }
    line
}

/// The seconds a part took, to two figures after the point.
fn secs(took: Duration) -> String {
    format!("{:.2}s", took.as_secs_f64())
}

/// The closing count, which says what the run covered rather than only whether
/// it was green. A run that examined less than the whole set says so here.
fn tail(results: &[Reported]) -> String {
    let mut ran = 0;
    let mut skipped = 0;
    let mut refused = 0;
    let mut unknown = 0;
    let mut not_reached = 0;
    for result in results {
        match result.outcome {
            Outcome::Ran(..) => ran += 1,
            Outcome::Skipped(_) => skipped += 1,
            Outcome::Failed(..) => refused += 1,
            Outcome::Unstartable(_) => unknown += 1,
            Outcome::NotReached => not_reached += 1,
        }
    }
    let verdict = match verdict(results) {
        Exit::Clean => "clean",
        Exit::Refused => "refused",
        Exit::CouldNotJudge => "could not judge",
    };
    format!(
        "\n{verdict}: {} part(s), {ran} ran, {skipped} skipped, {refused} refused, \
         {unknown} could not be started, {not_reached} not reached\n",
        results.len()
    )
}

/// The verdict of a run, from what its parts did.
///
/// A part that could not be started outranks a refusal, because a run holding
/// one of each did not judge the subject that part was about, and reporting
/// that as a plain refusal would say more than the run knows.
fn verdict(results: &[Reported]) -> Exit {
    if results
        .iter()
        .any(|result| matches!(result.outcome, Outcome::Unstartable(_)))
    {
        return Exit::CouldNotJudge;
    }
    if results
        .iter()
        .any(|result| matches!(result.outcome, Outcome::Failed(..)))
    {
        return Exit::Refused;
    }
    Exit::Clean
}

/// Finds the workspace root, walking up from `from`.
///
/// The root is the nearest directory at or above `from` whose `Cargo.toml`
/// declares a workspace. It is found rather than compiled in, because a path
/// baked in at build time is a path that is right on one machine.
#[must_use]
pub fn workspace_root(from: &Path) -> Option<PathBuf> {
    from.ancestors()
        .find(|directory| {
            std::fs::read_to_string(directory.join("Cargo.toml"))
                .is_ok_and(|manifest| declares_a_workspace(&manifest))
        })
        .map(Path::to_path_buf)
}

/// Whether the text of a manifest declares a workspace.
///
/// Reading for the table rather than parsing the manifest keeps the gate free
/// of a parser it would otherwise carry for one line, and the cost is named:
/// this reads a table header, so a manifest that writes the same table in
/// another legal spelling is not recognised.
fn declares_a_workspace(manifest: &str) -> bool {
    manifest
        .lines()
        .any(|manifest_line| manifest_line.trim_end() == "[workspace]")
}

#[cfg(test)]
mod tests {
    use super::*;

    fn ran(name: &'static str) -> Reported {
        Reported {
            name,
            examines: "what it examines",
            outcome: Outcome::Ran(Duration::ZERO, None),
        }
    }

    fn with(name: &'static str, outcome: Outcome) -> Reported {
        Reported {
            name,
            examines: "what it examines",
            outcome,
        }
    }

    // One test per exit code, because the three values are the interface and a
    // caller that cannot tell them apart eventually reads a broken invocation
    // as a clean refusal.

    #[test]
    fn a_run_where_nothing_refused_is_clean() {
        let results = [
            ran("format"),
            with("schema", Outcome::Skipped("a schema exists")),
        ];
        assert_eq!(verdict(&results), Exit::Clean);
        assert_eq!(verdict(&results).code(), 0);
    }

    #[test]
    fn a_run_where_a_part_refused_is_a_refusal() {
        let results = [
            ran("format"),
            with("lint", Outcome::Failed(Duration::ZERO, "said this".into())),
        ];
        assert_eq!(verdict(&results), Exit::Refused);
        assert_eq!(verdict(&results).code(), 1);
    }

    #[test]
    fn a_run_where_a_part_could_not_be_started_could_not_judge() {
        let results = [
            ran("format"),
            with("lint", Outcome::Unstartable("no such program".into())),
        ];
        assert_eq!(verdict(&results), Exit::CouldNotJudge);
        assert_eq!(verdict(&results).code(), 2);
    }

    #[test]
    fn a_part_that_could_not_be_started_outranks_a_refusal() {
        let results = [
            with(
                "format",
                Outcome::Failed(Duration::ZERO, "said this".into()),
            ),
            with("lint", Outcome::Unstartable("no such program".into())),
        ];
        assert_eq!(verdict(&results), Exit::CouldNotJudge);
    }

    #[test]
    fn a_skipped_part_says_what_would_make_it_run() {
        let printed = line(
            &with("schema", Outcome::Skipped("there is a schema to read")),
            6,
        );
        assert!(printed.contains("skipped"), "{printed}");
        assert!(
            printed.contains("runs when there is a schema to read"),
            "{printed}"
        );
    }

    #[test]
    fn a_failed_part_carries_everything_it_wrote() {
        let said = "error: this is the thing to fix";
        let printed = line(
            &with("lint", Outcome::Failed(Duration::ZERO, said.into())),
            6,
        );
        assert!(printed.contains(said), "{printed}");
    }

    #[test]
    fn a_part_after_a_failure_is_reported_as_not_reached() {
        let printed = line(&with("tests", Outcome::NotReached), 6);
        assert!(printed.contains("not reached"), "{printed}");
    }

    #[test]
    fn the_tail_counts_every_part_in_one_of_its_classes() {
        let results = [
            ran("format"),
            with("lint", Outcome::Failed(Duration::ZERO, String::new())),
            with("schema", Outcome::Skipped("a schema exists")),
            with("tests", Outcome::NotReached),
        ];
        let tail = tail(&results);
        assert!(tail.contains("4 part(s)"), "{tail}");
        assert!(tail.contains("1 ran"), "{tail}");
        assert!(tail.contains("1 skipped"), "{tail}");
        assert!(tail.contains("1 refused"), "{tail}");
        assert!(tail.contains("1 not reached"), "{tail}");
    }

    #[test]
    fn every_part_is_named_and_says_what_it_examines() {
        for part in PARTS {
            assert!(!part.name.is_empty());
            assert!(
                !part.examines.is_empty(),
                "{} says nothing about its subject",
                part.name
            );
            match part.runs {
                Runs::Command(argv) => assert!(!argv.is_empty(), "{} names no program", part.name),
                Runs::Check(_) => {}
                Runs::NotBuilt(would_run_when) => {
                    assert!(!would_run_when.is_empty(), "{} says nothing", part.name);
                }
            }
        }
    }

    #[test]
    fn a_package_manifest_is_not_the_workspace_root() {
        // The shape of a crate manifest in this workspace. The walk has to
        // pass one of these, or it stops at the first crate directory it is
        // run from and every part then examines that crate alone.
        let package = "[package]\nname = \"stoffbuch-gate\"\nedition.workspace = true\n";
        assert!(!declares_a_workspace(package));
    }

    #[test]
    fn the_workspace_manifest_is_the_root() {
        let workspace = "[workspace]\nmembers = [\"crates/*\"]\nresolver = \"3\"\n";
        assert!(declares_a_workspace(workspace));
    }

    // The line-endings check. The fixtures are byte strings with the endings
    // written as escapes, because a literal carriage return in this file would
    // be normalised on its way into the tree by the very declaration the check
    // exists to hold, and the fixture would then prove nothing.

    #[test]
    fn a_tabulated_block_with_line_feed_endings_is_clean() {
        let block = b"T\tn\n300\t3.42\n400\t3.44\n";
        assert!(!carries_a_carriage_return(block));
    }

    #[test]
    fn the_same_block_written_with_carriage_returns_is_refused() {
        // Byte for byte the block above as a converting checkout writes it.
        let block = b"T\tn\r\n300\t3.42\r\n400\t3.44\r\n";
        assert!(carries_a_carriage_return(block));
    }

    #[test]
    fn a_carriage_return_on_its_own_is_refused() {
        // Not every stray carriage return arrives in a pair. One inside a
        // field is what the tabulated block format refuses and has no quoting
        // rule to fall back on.
        assert!(carries_a_carriage_return(b"300\t3.42\ris not 3.42"));
    }

    #[test]
    fn a_carriage_return_at_the_very_end_is_refused() {
        assert!(carries_a_carriage_return(b"300\t3.42\n\r"));
    }

    #[test]
    fn a_carriage_return_past_the_first_eight_thousand_bytes_is_refused() {
        // The near-miss this check is most likely to be written with: the
        // binary test below reads a prefix, and reusing that prefix for the
        // search would pass a file whose only carriage return is late in it.
        let mut late = vec![b'x'; 9000];
        late.push(b'\r');
        assert!(carries_a_carriage_return(&late));
    }

    #[test]
    fn a_file_with_a_zero_byte_is_binary_and_is_not_judged() {
        assert!(is_binary(b"\x89PNG\r\n\x1a\n\x00\x00\x00\rIHDR"));
        assert!(!is_binary(b"T\tn\n300\t3.42\n"));
    }

    #[test]
    fn a_zero_byte_far_into_a_file_leaves_it_text() {
        // git decides on a prefix and so does this, so the two agree on which
        // files the declaration reaches. The cost is that a file with a zero
        // byte after the prefix is read as text, which is the direction that
        // examines more rather than less.
        let mut late = vec![b'x'; 9000];
        late.push(0);
        assert!(!is_binary(&late));
    }

    #[test]
    fn a_part_that_ran_prints_what_it_did_not_reach() {
        let note = "36 text file(s) read, 1 skipped as binary and not judged";
        let printed = line(
            &with(
                "line-endings",
                Outcome::Ran(Duration::ZERO, Some(note.to_owned())),
            ),
            12,
        );
        assert!(printed.contains("ran"), "{printed}");
        assert!(printed.contains(note), "{printed}");
    }

    // The headless check. Every fixture spelling below is assembled from two
    // halves rather than written out, and the elision is deliberate: this
    // module is test code, so it is exactly what the check reads, and a fixture
    // written as a literal would make the suite refuse itself. `concat!` puts
    // the halves together before anything runs, so what the check is given is
    // the spelling a contributor would actually write.

    fn as_written(first: &str, second: &str) -> String {
        format!("#[cfg(test)]\nmod tests {{\n    let a = {first}{second};\n}}\n")
    }

    #[test]
    fn a_test_reading_a_display_variable_is_refused() {
        let source = as_written(
            "std::env::var(\"",
            "DISP\
LAY\")",
        );
        let reaching = reached(&source, false);
        assert_eq!(reaching.len(), 1, "{reaching:?}");
        assert_eq!(reaching[0].0, 3);
        assert!(reaching[0].2.contains("display"), "{reaching:?}");
    }

    #[test]
    fn a_test_opening_a_socket_is_refused() {
        let source = as_written("std::net::Tcp", "Stream::connect(there)");
        let reaching = reached(&source, false);
        assert_eq!(reaching.len(), 1, "{reaching:?}");
        assert!(reaching[0].2.contains("socket"), "{reaching:?}");
    }

    #[test]
    fn a_test_asking_for_rights_is_refused() {
        let source = as_written("Command::new(\"su", "do\")");
        let reaching = reached(&source, false);
        assert_eq!(reaching.len(), 1, "{reaching:?}");
        assert!(
            reaching[0].2.contains("administrative rights"),
            "{reaching:?}"
        );
    }

    #[test]
    fn the_near_neighbour_of_each_one_passes() {
        // As little apart from the fixtures above as the surface allows. A
        // temporary directory rather than a display, a channel rather than a
        // socket, and a subprocess that is not a rights prompt. A check that
        // refused these would be one somebody weakens until it holds nothing.
        let neighbours = [
            "std::env::var(\"CARGO_MANIFEST_DIR\")",
            "std::sync::mpsc::channel()",
            "Command::new(\"git\").args([\"ls-files\"])",
        ];
        for neighbour in neighbours {
            let source = as_written(neighbour, "");
            assert!(reached(&source, false).is_empty(), "{neighbour}");
        }
    }

    #[test]
    fn production_code_above_the_test_module_is_not_judged() {
        // The command line reaches a terminal and the gate starts processes,
        // and neither is a test. The rule is about the suite, so the check
        // reads the suite.
        let source = format!(
            "fn main() {{\n    Command::new(\"su{}\");\n}}\n#[cfg(test)]\nmod tests {{}}\n",
            "do"
        );
        assert!(reached(&source, false).is_empty(), "{source}");
    }

    #[test]
    fn a_file_under_tests_is_read_whole() {
        // An integration test file has no `#[cfg(test)]` line to start at, so
        // a check that waited for one would read none of it.
        assert!(is_a_test_file("crates/stoffbuch/tests/reads_a_row.rs"));
        assert!(!is_a_test_file("crates/stoffbuch/src/lib.rs"));
        let source = format!("let a = std::net::Tcp{}::bind(here);\n", "Listener");
        assert!(
            reached(&source, false).is_empty(),
            "not test code without the marker"
        );
        assert_eq!(
            reached(&source, true).len(),
            1,
            "test code when the file is one"
        );
    }

    #[test]
    fn every_forbidden_surface_carries_the_rule_it_comes_from() {
        for forbidden in FORBIDDEN {
            assert!(!forbidden.spelling.is_empty());
            assert!(
                forbidden.rule.starts_with("a test may not"),
                "{} states no rule a contributor can act on",
                forbidden.spelling
            );
        }
    }

    #[test]
    fn a_table_whose_name_merely_starts_with_the_word_is_not_the_root() {
        // `[workspace.package]` is in the root manifest of this tree and is
        // also legal in a crate manifest, so a prefix match would name the
        // wrong directory as the root.
        assert!(!declares_a_workspace(
            "[workspace.package]\nedition = \"2024\"\n"
        ));
        assert!(!declares_a_workspace("[workspace.lints.rust]\n"));
    }
}
