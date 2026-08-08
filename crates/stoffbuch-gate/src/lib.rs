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
    /// It ran and refused nothing.
    Ran(Duration),
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
    /// Nothing yet. The part is declared anyway, so that a run cannot be read
    /// as covering what the part names, and the sentence says what would make
    /// it run.
    NotBuilt(&'static str),
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
        Ok(output) if output.status.success() => Outcome::Ran(took),
        Ok(output) => {
            let mut said = String::from_utf8_lossy(&output.stdout).into_owned();
            said.push_str(&String::from_utf8_lossy(&output.stderr));
            Outcome::Failed(took, said)
        }
    }
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
        Outcome::Ran(took) => {
            let _ = writeln!(
                line,
                "{name:width$}  ran      {:>7}  {}",
                secs(*took),
                result.examines
            );
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
            Outcome::Ran(_) => ran += 1,
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
            outcome: Outcome::Ran(Duration::ZERO),
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
