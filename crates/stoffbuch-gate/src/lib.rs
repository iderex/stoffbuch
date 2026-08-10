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
//!
//! Refusal surface. This file decides whether a subject is refused, which is
//! the code held to a higher standard than the rest of the tree, and
//! `docs/decisions/static-analysis-and-the-refusal-surface.md` is where that
//! is argued and where this line is given its meaning.

mod canonical;

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
pub(crate) enum Judged {
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
        name: "guards",
        examines: "every tracked text file, for a character that reorders what a reader sees, and the guards only the server runs",
        runs: Runs::Check(every_guard_is_accounted_for),
    },
    Part {
        name: "headless",
        examines: "the test code in every tracked Rust file, for a surface a test may not reach",
        runs: Runs::Check(a_test_reaches_nothing_it_may_not),
    },
    Part {
        name: "invariants",
        examines: "every tracked file a record names, for a rule no schema can express",
        runs: Runs::Check(no_invariant_is_broken),
    },
    Part {
        name: "surface",
        examines: "every tracked Rust file, for whether it says it decides refusals",
        runs: Runs::Check(the_refusal_surface_is_named),
    },
    Part {
        name: "findings",
        examines: "every tracked file under crates/ and .github/, for a finding turned off with no record",
        runs: Runs::Check(every_accepted_finding_is_recorded),
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
        name: "canonical",
        examines: "every tracked record under register/, against the form the register is stored in",
        runs: Runs::Check(canonical::every_record_is_in_the_canonical_form),
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
pub(crate) fn tracked(root: &Path) -> Result<Vec<String>, String> {
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

/// A run of characters that changes what a reader sees without changing what a
/// parser reads, with the rule it comes from and how the guard on the server
/// writes it.
///
/// The range is held here rather than a pattern, so that a refusal can print
/// the codepoint and the rule instead of an expression somebody has to decode,
/// and so that the character itself never has to appear in what a refusal
/// writes.
struct Reordering {
    /// The first codepoint of the run.
    first: char,
    /// The last codepoint of the run, which is `first` where the run is one
    /// character.
    last: char,
    /// What the run does, in words a contributor can act on.
    rule: &'static str,
}

impl Reordering {
    /// The run as the guard on the server spells it inside its pattern.
    ///
    /// Derived from the range rather than written beside it. Held as a second
    /// field it could disagree with the range it names, and then a run
    /// narrowed by one codepoint would still find its own spelling in the
    /// pattern and report the two guards as agreeing.
    fn spelled(&self) -> String {
        if self.first == self.last {
            format!("\\x{{{:04X}}}", u32::from(self.first))
        } else {
            format!(
                "\\x{{{:04X}}}-\\x{{{:04X}}}",
                u32::from(self.first),
                u32::from(self.last)
            )
        }
    }
}

/// Every run of characters this refuses.
///
/// It is the set the guard on the server carries, and the two are held
/// together below rather than left to agree by memory. Ordinary characters
/// outside ASCII are not in it and may not be: this register cites authors,
/// journals and materials whose names carry them, in exactly the fields a
/// citation depends on, so a guard that refused them would refuse half the
/// literature.
///
/// The byte order mark is deliberately absent, for the reason the guard on the
/// server writes beside its pattern. For a register record it is refused
/// anyway, by the canonical form, which requires UTF-8 with none.
const REORDERING: &[Reordering] = &[
    Reordering {
        first: '\u{202A}',
        last: '\u{202E}',
        rule: "an embedding or an override reorders every character after it until it is \
               popped, so a line reads one way and says another",
    },
    Reordering {
        first: '\u{2066}',
        last: '\u{2069}',
        rule: "an isolate reorders the run it encloses, and nothing in the rendered line \
               shows where that run began",
    },
    Reordering {
        first: '\u{200E}',
        last: '\u{200E}',
        rule: "a left-to-right mark changes the direction of what follows it and takes no \
               width, so a diff shows nothing",
    },
    Reordering {
        first: '\u{200F}',
        last: '\u{200F}',
        rule: "a right-to-left mark changes the direction of what follows it and takes no \
               width, so a diff shows nothing",
    },
    Reordering {
        first: '\u{061C}',
        last: '\u{061C}',
        rule: "an Arabic letter mark changes direction and takes no width",
    },
    Reordering {
        first: '\u{200B}',
        last: '\u{200D}',
        rule: "a zero width space, non-joiner or joiner splits or joins what renders as one \
               word, so two identifiers that look identical are not",
    },
    Reordering {
        first: '\u{2060}',
        last: '\u{2060}',
        rule: "a word joiner takes no width and has no use in a register file",
    },
];

/// Where the guard that runs on the server keeps the same set.
///
/// It is read rather than trusted. The name that job produces is required by
/// the protection rule on the default branch, so this file cannot replace it
/// and the two are copies; what stops a copy going stale is that one of them
/// refuses when the other moves.
const SERVER_SIDE_GUARD: &str = ".github/workflows/unicode-guard.yml";

/// A guard this repository has that this command does not run, and what
/// running it here would cost.
///
/// They are named so that a clean run cannot be read as a run that covered
/// them. Every one of them reads something no clone holds, which is why the
/// answer for each is the same shape: it stays where it is, and the reason is
/// here rather than in a document.
struct Elsewhere {
    /// The name its run carries, which is the only handle a protection rule
    /// has on it.
    name: &'static str,
    /// What it reads.
    reads: &'static str,
    /// What running it inside this command would cost.
    cost: &'static str,
}

/// Every guard that runs on the server and not here.
const ELSEWHERE: &[Elsewhere] = &[
    Elsewhere {
        name: "DCO sign-off",
        reads: "every commit of a change against the base it was opened on",
        cost: "a base nobody has named locally, so a run here would judge whatever this \
               clone last fetched and call it the change",
    },
    Elsewhere {
        name: "dependency-review",
        reads: "what a change adds to the dependency graph, as the forge computes it for a \
                base and a head",
        cost: "a call out to that forge on every run, and a credential to make it with, \
               neither of which this command has or wants",
    },
    Elsewhere {
        name: "Audit workflows (zizmor)",
        reads: "the workflow files, for the ways a job can be made to do what it was not \
                asked to",
        cost: "an analyser this tree does not carry, so a second toolchain pinned and \
               installed before the gate could start",
    },
    Elsewhere {
        name: "Scorecard analysis",
        reads: "the repository's own settings and history, which live at the forge rather \
                than in the tree",
        cost: "the same call out and credential as the dependency review, over state no \
               clone holds",
    },
];

/// Refuses a tracked text file carrying a character that reorders what a
/// reader sees, and names the guards this command did not run.
///
/// The register is the reason this is not only a source code rule. A citation
/// is text a person reads and a locator is text a person follows, and a
/// character that reorders a rendered line makes the page number a reader sees
/// different from the one the file carries. That failure is invisible in a
/// diff, which is where curation is reviewed.
///
/// It refuses the set and nothing wider. Ordinary characters outside ASCII
/// pass, because the fields a citation depends on are full of them, and the
/// pair of fixtures in the suite holds both directions rather than a comment
/// asserting them.
///
/// The other four guards are not run and are named instead. Each reads
/// something that exists at the forge rather than in a clone, so bringing one
/// here would mean judging a substitute for its subject, which is worse than
/// saying plainly that it ran elsewhere.
fn every_guard_is_accounted_for(root: &Path) -> Judged {
    let tracked = match tracked(root) {
        Ok(tracked) => tracked,
        Err(why) => return Judged::CouldNotJudge(why),
    };

    let Ok(server) = std::fs::read_to_string(root.join(SERVER_SIDE_GUARD)) else {
        return Judged::Refused(format!(
            "{SERVER_SIDE_GUARD} could not be read, so the check whose name the protection \
             rule on the default branch requires is not in this tree. A rule pointing at a \
             name nothing produces passes everything.\n"
        ));
    };
    let adrift = adrift_from(&server);
    if !adrift.is_empty() {
        let mut why = format!(
            "{} run(s) refused here are not in the pattern {SERVER_SIDE_GUARD} uses, so the \
             two guards disagree about what they refuse:\n\n",
            adrift.len()
        );
        for run in &adrift {
            let _ = writeln!(why, "  {}\n      {}", run.spelled(), run.rule);
        }
        let _ = writeln!(
            why,
            "\nThe set is one thing held in two places because the name that job produces is \
             required for a merge. Move both or neither."
        );
        return Judged::Refused(why);
    }

    let mut offences = Vec::new();
    let mut read = 0_usize;
    let mut binary = 0_usize;
    let mut undecodable = Vec::new();

    for name in tracked {
        let Ok(bytes) = std::fs::read(root.join(&name)) else {
            continue;
        };
        if is_binary(&bytes) {
            binary += 1;
            continue;
        }
        let Ok(text) = std::str::from_utf8(&bytes) else {
            // Not UTF-8, so its characters cannot be read and nothing about it
            // is known. Named rather than counted, because a file this check
            // did not judge is the one place it fails open.
            undecodable.push(name);
            continue;
        };
        read += 1;
        for (line, character, run) in reorders(text) {
            offences.push(format!(
                "  {name}:{line}  U+{:04X}\n      {}",
                u32::from(character),
                run.rule
            ));
        }
    }

    if !offences.is_empty() {
        let mut why = format!(
            "{} character(s) that reorder what a reader sees:\n\n",
            offences.len()
        );
        for offence in &offences {
            let _ = writeln!(why, "{offence}");
        }
        let _ = writeln!(
            why,
            "\nThe codepoint is printed and the character is not, so this refusal can be \
             quoted into an issue without carrying the thing it is about."
        );
        return Judged::Refused(why);
    }

    let mut note = format!("{read} text file(s) read");
    if binary > 0 {
        let _ = write!(note, ", {binary} skipped as binary and not judged");
    }
    for name in &undecodable {
        let _ = write!(note, ", {name} is not UTF-8 and was not judged");
    }
    let _ = write!(
        note,
        "\n{} guard(s) run at the forge and not here, and what running each here would cost:",
        ELSEWHERE.len()
    );
    for guard in ELSEWHERE {
        let _ = write!(
            note,
            "\n  {}\n    reads {}\n    would cost {}",
            guard.name, guard.reads, guard.cost
        );
    }
    Judged::Nothing(Some(note))
}

/// Every run this refuses that the text of the server-side guard does not
/// name.
///
/// The comparison is over the spelling the pattern uses rather than over a
/// parse of it, because a parser for one pattern dialect is a third place the
/// set could be wrong, and what is wanted here is only whether the two say the
/// same thing.
///
/// Only the line that assigns the pattern is read. The prose around it names
/// several of these codepoints in another spelling, and reading the whole file
/// would let a comment stand in for the pattern, which is the direction that
/// fails open. A file with no such line leaves every run adrift and is
/// refused.
fn adrift_from(server: &str) -> Vec<&'static Reordering> {
    let pattern = server
        .lines()
        .find(|server_line| server_line.trim_start().starts_with("pattern="))
        .unwrap_or_default();
    REORDERING
        .iter()
        .filter(|run| !pattern.contains(&run.spelled()))
        .collect()
}

/// Every reordering character in `text`, with the line it is on and the run it
/// belongs to.
///
/// The line is counted from one, and each occurrence is reported rather than
/// each file, because the repair is per occurrence and a count of files would
/// leave a reader hunting.
fn reorders(text: &str) -> Vec<(usize, char, &'static Reordering)> {
    let mut found = Vec::new();
    for (index, text_line) in text.lines().enumerate() {
        for character in text_line.chars() {
            if let Some(run) = REORDERING
                .iter()
                .find(|run| character >= run.first && character <= run.last)
            {
                found.push((index + 1, character, run));
            }
        }
    }
    found
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

    for name in tracked.iter().filter(|name| is_rust(name)) {
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

/// Whether a tracked path is a Rust source file.
fn is_rust(name: &str) -> bool {
    Path::new(name)
        .extension()
        .is_some_and(|extension| extension.eq_ignore_ascii_case("rs"))
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

/// Where the invariant records live.
///
/// A record carries the spelling it forbids, so this directory is a set of
/// files each containing the thing it exists to refuse. Nothing under it is
/// ever searched, and a record naming it as a subject is refused rather than
/// quietly skipped.
const INVARIANTS: &str = "docs/invariants/";

/// One rule the tree holds by searching itself, as a record declares it.
#[derive(Debug, PartialEq, Eq)]
struct Invariant {
    /// The name a refusal prints, taken from the file name.
    id: String,
    /// The path prefix whose tracked files are searched.
    subject: String,
    /// Text that may not appear in the subject, lowercased at load, because
    /// the search ignores case: the same mistake is written in three casings
    /// and listing all three would be a list of spellings for one spelling.
    spellings: Vec<String>,
    /// The sentence a refusal prints, in words somebody can act on without
    /// opening the record.
    rule: String,
}

/// Refuses a tracked file that breaks an invariant a record declares.
///
/// The records are the authority for which invariants exist and this function
/// supplies only the search, so adding one is adding a file. What the records
/// are is printed by the run and listed in no document.
///
/// A record that declares itself held while naming no spelling, no subject or
/// no rule is refused rather than passed over, because such a record holds
/// nothing while reading as though it does, and that is the one failure a
/// register of rules cannot afford.
fn no_invariant_is_broken(root: &Path) -> Judged {
    let tracked = match tracked(root) {
        Ok(tracked) => tracked,
        Err(why) => return Judged::CouldNotJudge(why),
    };

    let declared: Vec<&String> = tracked
        .iter()
        .filter(|name| name.starts_with(INVARIANTS) && is_a_record(name))
        .collect();

    let mut invariants = Vec::new();
    let mut unheld = 0_usize;
    for name in &declared {
        let text = match std::fs::read_to_string(root.join(name)) {
            Ok(text) => text,
            Err(why) => return Judged::CouldNotJudge(format!("{name}: {why}")),
        };
        match read_invariant(name, &text) {
            Ok(Some(invariant)) => invariants.push(invariant),
            Ok(None) => unheld += 1,
            Err(why) => {
                return Judged::Refused(format!("{name} is not a usable record.\n\n{why}\n"));
            }
        }
    }

    let mut broken = Vec::new();
    let mut searched = 0_usize;
    for name in &tracked {
        let reaching: Vec<&Invariant> = invariants
            .iter()
            .filter(|invariant| name.starts_with(&invariant.subject))
            .collect();
        if reaching.is_empty() {
            continue;
        }
        let Ok(text) = std::fs::read_to_string(root.join(name)) else {
            continue;
        };
        searched += 1;
        for (line, invariant) in breaks(&text, &reaching) {
            broken.push(format!(
                "  {name}:{line}  {}\n      {}",
                invariant.id, invariant.rule
            ));
        }
    }

    if broken.is_empty() {
        return Judged::Nothing(Some(format!(
            "{} held, {unheld} recorded as not holdable this way, {searched} file(s) searched",
            invariants.len()
        )));
    }
    let mut why = format!("{} line(s) break an invariant:\n\n", broken.len());
    for one in &broken {
        let _ = writeln!(why, "{one}");
    }
    let _ = writeln!(
        why,
        "The rule under each line is the whole of it. Where the rule is wrong \
         the repair is\nto the record in {INVARIANTS}, argued there, and not a \
         spelling removed to get a\ngreen run."
    );
    Judged::Refused(why)
}

/// Whether a file in the records directory is a record.
///
/// The readme there argues the shape and is not one, so it is named rather than
/// detected: a record with no header lines and a readme with none are the same
/// file to a reader that only counts fields, and one of them is a defect.
fn is_a_record(name: &str) -> bool {
    Path::new(name)
        .extension()
        .is_some_and(|extension| extension.eq_ignore_ascii_case("md"))
        && !name.ends_with("README.md")
}

/// Every line of `text` that breaks one of `invariants`, with the line number.
///
/// The comparison is over lowercased text, so one spelling covers the snake,
/// screaming and camel casings of the same mistake. It is a substring match
/// rather than a word match, deliberately: the mistake this is for arrives as
/// part of a longer name at least as often as on its own.
fn breaks<'a>(text: &str, invariants: &[&'a Invariant]) -> Vec<(usize, &'a Invariant)> {
    let mut broken = Vec::new();
    for (index, line) in text.lines().enumerate() {
        let lowered = line.to_ascii_lowercase();
        for invariant in invariants {
            if invariant
                .spellings
                .iter()
                .any(|spelling| lowered.contains(spelling))
            {
                broken.push((index + 1, *invariant));
            }
        }
    }
    broken
}

/// One record read, or the reason it is not a usable one.
///
/// `Ok(None)` is a record declaring itself not holdable by a search. That is a
/// legitimate state and the record says why in its body, so the run counts it
/// rather than passing it over in silence.
fn read_invariant(name: &str, text: &str) -> Result<Option<Invariant>, String> {
    let mut held = None;
    let mut subject = None;
    let mut rule = None;
    let mut spellings = Vec::new();
    for line in text.lines() {
        let Some((key, value)) = line.split_once(':') else {
            continue;
        };
        let value = value.trim().to_owned();
        match key {
            "Held" => held = Some(value == "yes"),
            "Subject" => subject = Some(value),
            "Rule" => rule = Some(value),
            "Spelling" => spellings.push(value.to_ascii_lowercase()),
            _ => {}
        }
    }

    let id = name
        .rsplit('/')
        .next()
        .unwrap_or(name)
        .trim_end_matches(".md")
        .to_owned();

    match held {
        None => {
            return Err(
                "It carries no Held line, so whether the gate searches for it is not stated."
                    .to_owned(),
            );
        }
        Some(false) => return Ok(None),
        Some(true) => {}
    }

    let (Some(subject), Some(rule)) = (subject, rule) else {
        return Err(
            "It declares Held: yes and carries no Subject or no Rule, so it names nothing to \
             search and prints nothing a refusal could act on."
                .to_owned(),
        );
    };
    if spellings.is_empty() {
        return Err(
            "It declares Held: yes and lists no Spelling, so it would pass every tree while \
             reading as an invariant that holds."
                .to_owned(),
        );
    }
    if INVARIANTS.starts_with(&subject) || subject.starts_with(INVARIANTS) {
        return Err(format!(
            "Its subject reaches {INVARIANTS}, where every record carries the spelling it \
             forbids. A search there refuses the records themselves."
        ));
    }

    Ok(Some(Invariant {
        id,
        subject,
        spellings,
        rule,
    }))
}

/// The line in a file's own module documentation that puts it in the refusal
/// surface.
const SURFACE: &str = "Refusal surface.";

/// The spelling a refusal in this tree is written with.
///
/// There is one, because there is one way to refuse a subject. A second
/// spelling would be a second surface, and nothing would be naming it.
const REFUSAL: &str = "Judged::Refused";

/// Refuses a file that decides a refusal without saying so, and a file that
/// says so and decides none.
///
/// The code that decides whether a subject is refused is what this project
/// holds to a higher standard than the rest, because a check that fails open
/// lets a wrong number into the register and every result computed from that
/// row inherits it. Naming that code is what a coverage bar and a mutation run
/// are pointed at, and this part prints the names rather than a document
/// holding a list somebody has to remember to update.
///
/// Both directions are refused. An unmarked file that decides a refusal is a
/// piece of the surface nothing downstream would reach; a marked file that
/// decides none widens the surface until it names most of the tree, and a
/// standard that covers everything is one nobody meets.
///
/// The limits are stated in
/// `docs/decisions/static-analysis-and-the-refusal-surface.md` and are real:
/// this names a file rather than a function, and it reads text, so a file that
/// merely matches on a refusal is asked for the line too. Both of those name
/// more of the surface rather than less, which is the direction that hides no
/// hole.
fn the_refusal_surface_is_named(root: &Path) -> Judged {
    let tracked = match tracked(root) {
        Ok(tracked) => tracked,
        Err(why) => return Judged::CouldNotJudge(why),
    };
    let Survey {
        surface,
        disagreeing,
        read,
    } = survey_the_surface(root, &tracked);

    if !disagreeing.is_empty() {
        let mut why = format!(
            "{} file(s) disagree with what they say about the refusal surface:\n\n",
            disagreeing.len()
        );
        for one in &disagreeing {
            let _ = writeln!(why, "{one}");
        }
        let _ = writeln!(
            why,
            "\nA file that decides a refusal says so in its own module documentation, in \
             a\nline reading `{SURFACE}`. What that line means and what it cannot do are \
             in\ndocs/decisions/static-analysis-and-the-refusal-surface.md. The scope a \
             coverage\nbar and a mutation run are given is what this part prints, so a file \
             missing\nfrom it is a file neither of them ever reaches."
        );
        return Judged::Refused(why);
    }

    let note = if surface.is_empty() {
        format!("{read} tracked Rust file(s) read, none in the refusal surface")
    } else {
        format!(
            "{read} tracked Rust file(s) read, {} in the refusal surface: {}",
            surface.len(),
            surface.join(" ")
        )
    };
    Judged::Nothing(Some(note))
}

/// What one walk over the tracked Rust files found about the surface.
struct Survey {
    /// The files that name themselves part of it and decide a refusal.
    surface: Vec<String>,
    /// The files whose documentation and code say different things, each with
    /// which of the two directions it went.
    disagreeing: Vec<String>,
    /// How many files were read, so a count in the report is not inferred from
    /// how many the tree happens to hold.
    read: usize,
}

/// Reads every tracked Rust file and places it against the surface marker.
///
/// The walk is here rather than inside the check because two callers need the
/// same answer: the check, which refuses a disagreement, and the command that
/// prints the surface for a coverage bar or a mutation run to be pointed at. A
/// second walk would be a second definition of what the surface is, and the
/// one that drifts is always the one nobody runs.
fn survey_the_surface(root: &Path, tracked: &[String]) -> Survey {
    let mut disagreeing = Vec::new();
    let mut surface = Vec::new();
    let mut read = 0_usize;

    for name in tracked.iter().filter(|name| is_rust(name)) {
        let Ok(text) = std::fs::read_to_string(root.join(name)) else {
            continue;
        };
        read += 1;
        match names_itself(&text) {
            (true, true) => surface.push(name.clone()),
            (false, true) => disagreeing.push(format!(
                "  {name}\n      decides a refusal and its module documentation does not say so"
            )),
            (true, false) => disagreeing.push(format!(
                "  {name}\n      says it decides refusals and decides none"
            )),
            (false, false) => {}
        }
    }

    Survey {
        surface,
        disagreeing,
        read,
    }
}

/// Every tracked file that names itself part of the refusal surface.
///
/// This is what a coverage bar and a mutation run are pointed at, and it is
/// derived from the same marker the `surface` part of the gate reads rather
/// than from a list in a configuration file. A list would be a second home for
/// a set that moves whenever a file starts or stops deciding refusals, and the
/// run that used the stale copy would report a clean score over code it never
/// touched.
///
/// # Errors
///
/// Returns the reason the tracked list could not be had. A caller that took an
/// empty list for an answer would point a run at nothing and read the result as
/// a surface with no survivors.
pub fn refusal_surface(root: &Path) -> Result<Vec<String>, String> {
    let tracked = tracked(root)?;
    Ok(survey_the_surface(root, &tracked).surface)
}

/// Whether a file's module documentation names it as part of the refusal
/// surface, and whether its own code decides a refusal.
///
/// What is read is everything above the first `#[cfg(test)]`, which is where
/// this repository puts a test module. A fixture is a refusal written down to
/// be read rather than one a run acts on, and a suite that put itself in the
/// surface would make the surface mostly suite.
///
/// A comment mentioning the spelling is not a decision, so only lines that are
/// not comments are read for it. That is what lets this file argue about a
/// refusal in prose without every such file joining the surface.
fn names_itself(text: &str) -> (bool, bool) {
    let mut named = false;
    let mut decides = false;
    for raw in text.lines() {
        let line = raw.trim_start();
        if line.starts_with("#[cfg(test)]") {
            break;
        }
        if line.starts_with("//!") {
            named |= line.contains(SURFACE);
        } else if !line.starts_with("//") {
            decides |= line.contains(REFUSAL);
        }
    }
    (named, decides)
}

/// Where a finding accepted rather than fixed is recorded.
const ACCEPTED: &str = "docs/accepted-findings/";

/// The spellings that turn a finding off rather than fixing it.
///
/// Each is assembled from two halves. This file sits under a subject the check
/// reads, so a spelling written whole here would be a suppression naming no
/// record, inside the check that refuses one.
const SUPPRESSIONS: &[&str] = &[
    concat!("#[all", "ow("),
    concat!("#[exp", "ect("),
    concat!("zizmor:", " ignore"),
];

/// Refuses a suppression that names no record, and a record no suppression
/// names.
///
/// A finding an analyser raises is either fixed or accepted, and an accepted
/// one is an argument that has to outlive the tool that raised it. A dismissal
/// held in a tool's own interface goes when the tool goes, and what a later
/// reader needs is not the dismissal but the reason.
///
/// It fails closed in both directions, so the register describes the tree
/// rather than its own history: a suppression with no record is a finding
/// turned off with the reason nowhere, and a record no suppression names is a
/// reason for something that is no longer there.
///
/// The subject is tracked files under `crates/` and `.github/`. The records
/// themselves are not read, because a record is about a spelling and carries
/// it, and a search there would refuse the records. The residual is that a
/// suppression written into a document is invisible here.
fn every_accepted_finding_is_recorded(root: &Path) -> Judged {
    let tracked = match tracked(root) {
        Ok(tracked) => tracked,
        Err(why) => return Judged::CouldNotJudge(why),
    };

    let records: Vec<&str> = tracked
        .iter()
        .filter(|name| name.starts_with(ACCEPTED) && is_a_record(name))
        .map(String::as_str)
        .collect();

    let mut unrecorded = Vec::new();
    let mut named = Vec::new();
    let mut read = 0_usize;

    for name in tracked
        .iter()
        .filter(|name| is_searched_for_a_finding(name))
    {
        let Ok(text) = std::fs::read_to_string(root.join(name)) else {
            continue;
        };
        read += 1;
        for (line, spelling, said) in suppressions(&text) {
            if let Some(record) = record_named(said, &records) {
                named.push(record);
            } else {
                unrecorded.push(format!(
                    "  {name}:{line}  {spelling}\n      names no record under {ACCEPTED}"
                ));
            }
        }
    }

    let dangling: Vec<&str> = records
        .iter()
        .copied()
        .filter(|record| !named.contains(record))
        .collect();

    if !unrecorded.is_empty() || !dangling.is_empty() {
        let mut why = String::new();
        if !unrecorded.is_empty() {
            let _ = writeln!(
                why,
                "{} finding(s) turned off with the reason nowhere:\n",
                unrecorded.len()
            );
            for one in &unrecorded {
                let _ = writeln!(why, "{one}");
            }
        }
        for record in &dangling {
            let _ = writeln!(
                why,
                "{record} accepts a finding that nothing suppresses any more."
            );
        }
        let _ = writeln!(
            why,
            "\nA suppression names its record on the same line, and {ACCEPTED}README.md \
             is\nwhere the shape of one is written. Lowering a lint for the whole tree is \
             a\nchange to the workspace manifest instead, argued there."
        );
        return Judged::Refused(why);
    }

    Judged::Nothing(Some(format!(
        "{read} file(s) searched, {} suppression(s), {} record(s)",
        named.len(),
        records.len()
    )))
}

/// Whether a tracked path is read for a suppression.
fn is_searched_for_a_finding(name: &str) -> bool {
    name.starts_with("crates/") || name.starts_with(".github/")
}

/// Every suppression in `text`, with its line, the spelling that named it and
/// the whole of the line it is on.
fn suppressions(text: &str) -> Vec<(usize, &'static str, &str)> {
    let mut found = Vec::new();
    for (index, line) in text.lines().enumerate() {
        if let Some(spelling) = SUPPRESSIONS
            .iter()
            .find(|spelling| line.contains(**spelling))
        {
            found.push((index + 1, *spelling, line));
        }
    }
    found
}

/// The record a suppression line names, where it names one that is in the tree.
///
/// The whole path is named rather than an identifier, so that the line a person
/// reads next to the suppression is the line they can open.
fn record_named<'a>(said: &str, records: &[&'a str]) -> Option<&'a str> {
    records.iter().copied().find(|record| said.contains(record))
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
                // Indented a line at a time. A note that says what a part did
                // not reach can carry more than one thing, and a second line
                // left at column zero reads as a new part rather than as more
                // of this one.
                for note_line in note.lines() {
                    let _ = writeln!(line, "{:width$}  {note_line}", "");
                }
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
    // Counted in an unsigned type on purpose. These only ever go up, and left
    // to be inferred they are signed, so a negated increment produces a
    // plausible line instead of stopping the run. A mutation run found exactly
    // that: the tail printed `-1 ran` and the test that covers this function
    // could not tell the count from its negation.
    let mut ran = 0_usize;
    let mut skipped = 0_usize;
    let mut refused = 0_usize;
    let mut unknown = 0_usize;
    let mut not_reached = 0_usize;
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
        // One part per class, and the whole line compared. This test used to
        // ask whether the line contained `1 ran`, which `-1 ran` also contains,
        // so the count could be negated and stay green: a mutation run over
        // this file found it, and the accounting is the property the whole
        // report rests on.
        let results = [
            ran("format"),
            with("lint", Outcome::Failed(Duration::ZERO, String::new())),
            with("schema", Outcome::Skipped("a schema exists")),
            with("tests", Outcome::NotReached),
            with("clippy", Outcome::Unstartable("no such program".to_owned())),
        ];
        assert_eq!(
            tail(&results),
            "\ncould not judge: 5 part(s), 1 ran, 1 skipped, 1 refused, \
             1 could not be started, 1 not reached\n"
        );
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

    // The guards part. Both fixtures are written as source rather than as
    // files, and the reason is the one
    // `docs/decisions/fixtures-and-what-a-test-may-be-about.md` gives for the
    // line-endings pair above: a file carrying the character this refuses
    // could not sit in the tree, because the guard on the server refuses it
    // and so does this. The escape is what the file holds; the character only
    // ever exists after the compiler has read it.

    /// A source in the shape of the ones this register cites.
    ///
    /// Invented, as every fixture here is, and carrying the characters a real
    /// one carries: an umlaut, a ring, a diaeresis and a micro sign, in the
    /// author list, the journal title and the locator. Those are the fields a
    /// citation depends on, and a guard that refused them would refuse most of
    /// the literature this register is for.
    const A_CITATION: &str = "Müller and Ångström, Zeitschrift für Physik 234 (1970) 56, \
                              Table 2, column 3, at 0.5 µm";

    /// What a walk found, written as codepoints, so that a failing test does
    /// not print the character it is about into a terminal or a transcript.
    fn codepoints(found: &[(usize, char, &Reordering)]) -> String {
        found
            .iter()
            .map(|(at, character, _)| format!("line {at}: U+{:04X}", u32::from(*character)))
            .collect::<Vec<String>>()
            .join(", ")
    }

    #[test]
    fn a_citation_carrying_ordinary_characters_outside_ascii_is_clean() {
        assert!(
            reorders(A_CITATION).is_empty(),
            "the fields a citation depends on carry these and must pass"
        );
    }

    #[test]
    fn the_same_citation_with_one_override_in_it_is_refused() {
        // The near neighbour of the fixture above, differing by one character
        // that takes no width. Rendered, the two lines can look identical; the
        // page number a reader follows is not the one the file carries.
        let deceived = A_CITATION.replace("Table 2", "Table \u{202E}2");
        let found = reorders(&deceived);
        assert_eq!(found.len(), 1, "{}", codepoints(&found));
        assert_eq!(found[0].1, '\u{202E}');
    }

    #[test]
    fn a_zero_width_space_inside_an_identifier_is_refused() {
        // The mistake with no visible trace at all: two identifiers that
        // render the same and are not equal.
        let found = reorders("  \"identifier\": \"si\u{200B}c-4h-0001\"");
        assert_eq!(found.len(), 1, "{}", codepoints(&found));
        assert_eq!(found[0].1, '\u{200B}');
    }

    #[test]
    fn every_run_this_refuses_is_refused_at_both_of_its_ends() {
        // A range written with its ends the wrong way round, or narrowed by
        // one, is a run this stops refusing while every other test stays
        // green.
        for run in REORDERING {
            for end in [run.first, run.last] {
                let found = reorders(&format!("a{end}b"));
                assert_eq!(found.len(), 1, "U+{:04X} is not refused", u32::from(end));
                assert_eq!(found[0].1, end);
            }
        }
    }

    #[test]
    fn a_refusal_carries_the_line_it_is_on() {
        let found = reorders("first\nsecond\nthird\u{2066}\n");
        assert_eq!(found.len(), 1, "{}", codepoints(&found));
        assert_eq!(found[0].0, 3);
    }

    /// The guard on the server, as this tree holds it.
    fn the_server_side_guard() -> String {
        let path = Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("..")
            .join("..")
            .join(SERVER_SIDE_GUARD);
        std::fs::read_to_string(path).expect("the guard on the server is in this tree")
    }

    #[test]
    fn the_guard_on_the_server_names_every_run_this_one_refuses() {
        // The set comparison rather than a list somebody remembers to update.
        // The name that job produces is required for a merge, so this file
        // cannot replace it, and two copies of one set are only safe while one
        // of them refuses when the other moves.
        let adrift = adrift_from(&the_server_side_guard());
        let adrift: Vec<String> = adrift.iter().map(|run| run.spelled()).collect();
        assert_eq!(adrift, Vec::<String>::new());
    }

    #[test]
    fn a_pattern_that_lost_one_run_is_refused() {
        // The near miss: one range deleted from the pattern and everything
        // else left alone, which is what an edit that meant to remove a false
        // positive actually looks like.
        let narrowed = the_server_side_guard().replace("\\x{200B}-\\x{200D}", "");
        let adrift = adrift_from(&narrowed);
        assert_eq!(adrift.len(), 1);
        assert_eq!(adrift[0].spelled(), "\\x{200B}-\\x{200D}");
    }

    #[test]
    fn a_guard_with_no_pattern_line_leaves_every_run_adrift() {
        // Fails closed. A file that was rewritten into a shape this cannot
        // read is not a file this may pass.
        assert_eq!(adrift_from("").len(), REORDERING.len());
    }

    #[test]
    fn every_guard_that_runs_at_the_forge_says_what_running_it_here_would_cost() {
        for guard in ELSEWHERE {
            assert!(!guard.name.is_empty());
            assert!(
                !guard.reads.is_empty(),
                "{} says nothing it reads",
                guard.name
            );
            assert!(!guard.cost.is_empty(), "{} names no cost", guard.name);
        }
    }

    #[test]
    fn a_note_of_several_lines_is_indented_a_line_at_a_time() {
        // A second line left at column zero reads as another part rather than
        // as more of this one, and the accounting is what the whole report
        // rests on.
        let printed = line(
            &with(
                "guards",
                Outcome::Ran(Duration::ZERO, Some("first\nsecond".to_owned())),
            ),
            6,
        );
        assert_eq!(printed.lines().count(), 3, "{printed}");
        for printed_line in printed.lines().skip(1) {
            assert!(printed_line.starts_with("        "), "{printed_line:?}");
        }
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

    // The invariants check. Every fixture below is assembled with `concat!`
    // for the same reason the headless fixtures are: this file is under
    // `crates/`, which is a subject a record names, so a fixture written as a
    // literal would break the invariant it is a fixture for.

    /// One held invariant's two fixtures.
    ///
    /// The pair is the whole point. A line that trips the invariant proves it
    /// bites; a near neighbour that does not proves it has not been widened
    /// until it refuses ordinary work, which is the direction this kind of
    /// check actually fails in.
    struct Fixtures {
        id: &'static str,
        trips: &'static str,
        near_neighbour: &'static str,
    }

    const FIXTURES: &[Fixtures] = &[Fixtures {
        id: "no-default-for-an-absent-condition",
        trips: concat!("    let t = assumed_", "temperature(row);"),
        near_neighbour: concat!("    let t = stated_", "temperature(row);"),
    }];

    /// Every record in this tree, read the way the check reads them.
    fn the_records() -> Vec<(String, Result<Option<Invariant>, String>)> {
        let root = Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("..")
            .join("..")
            .join(INVARIANTS);
        let mut read = Vec::new();
        for entry in std::fs::read_dir(&root).expect("the records directory is in this tree") {
            let path = entry.expect("a readable entry").path();
            let name = path
                .file_name()
                .and_then(|name| name.to_str())
                .unwrap_or_default()
                .to_owned();
            if !is_a_record(&name) {
                continue;
            }
            let text = std::fs::read_to_string(&path).expect("a readable record");
            let held = read_invariant(&name, &text);
            read.push((name, held));
        }
        read
    }

    #[test]
    fn every_held_invariant_has_a_fixture_that_trips_it_and_a_near_neighbour() {
        // The set comparison is what makes the obligation follow the record
        // rather than somebody's memory: a record added with no fixtures
        // reddens here, and a fixture left behind by a deleted record does
        // too.
        let mut held: Vec<String> = the_records()
            .into_iter()
            .filter_map(|(name, read)| {
                read.unwrap_or_else(|why| panic!("{name} is not a usable record: {why}"))
                    .map(|invariant| invariant.id)
            })
            .collect();
        held.sort();
        let mut fixtured: Vec<String> = FIXTURES.iter().map(|f| f.id.to_owned()).collect();
        fixtured.sort();
        assert_eq!(
            held, fixtured,
            "a held invariant with no fixtures, or the other way about"
        );
    }

    #[test]
    fn each_fixture_trips_its_own_invariant_and_its_neighbour_trips_nothing() {
        let records: Vec<Invariant> = the_records()
            .into_iter()
            .filter_map(|(_, read)| read.ok().flatten())
            .collect();
        let all: Vec<&Invariant> = records.iter().collect();
        for fixture in FIXTURES {
            let broken = breaks(fixture.trips, &all);
            assert_eq!(broken.len(), 1, "{}: {:?}", fixture.id, broken.len());
            assert_eq!(
                broken[0].1.id, fixture.id,
                "the refusal names the wrong invariant"
            );
            assert!(
                !broken[0].1.rule.is_empty(),
                "a refusal that prints no rule is one nobody can act on"
            );
            assert!(
                breaks(fixture.near_neighbour, &all).is_empty(),
                "{} refuses its near neighbour",
                fixture.id
            );
        }
    }

    #[test]
    fn a_record_holding_nothing_while_reading_as_though_it_holds_is_refused() {
        let missing_spelling = "Id: x\nHeld: yes\nSubject: crates/\nRule: a rule\n\nbody\n";
        let why = read_invariant("x.md", missing_spelling).expect_err("a record holding nothing");
        assert!(why.contains("Spelling"), "{why}");

        let missing_rule = "Id: x
Held: yes
Subject: crates/
Spelling: q

body
";
        assert!(read_invariant("x.md", missing_rule).is_err());

        let missing_held = "Id: x\nSubject: crates/\nRule: a rule\n\nbody\n";
        assert!(read_invariant("x.md", missing_held).is_err());
    }

    #[test]
    fn a_record_searching_the_records_is_refused() {
        // A record carries the spelling it forbids, so a subject reaching them
        // refuses the records themselves. The exclusion is stated by a refusal
        // rather than by a silent skip, which is why this is an error and not
        // an empty result.
        let reaching =
            format!("Id: x\nHeld: yes\nSubject: {INVARIANTS}\nRule: a rule\nSpelling: q\n\nbody\n");
        let why = read_invariant("x.md", &reaching).expect_err("a record searching the records");
        assert!(why.contains(INVARIANTS), "{why}");

        let wider = "Id: x\nHeld: yes\nSubject: docs/\nRule: a rule\nSpelling: q\n\nbody\n";
        assert!(
            read_invariant("x.md", wider).is_err(),
            "a subject the records sit under reaches them too"
        );
    }

    #[test]
    fn a_record_that_cannot_be_held_this_way_is_counted_rather_than_refused() {
        // Recording an invariant as unheld is a legitimate state and the point
        // of the register: the alternative is a pattern that half works, which
        // makes a green run mean something it does not.
        let unheld = "Id: x\nHeld: no\nRule: a rule\nRetired-by: something\n\nwhy not\n";
        assert_eq!(read_invariant("x.md", unheld), Ok(None));
    }

    #[test]
    fn the_search_ignores_case_and_does_not_require_a_word_boundary() {
        let records: Vec<Invariant> = the_records()
            .into_iter()
            .filter_map(|(_, read)| read.ok().flatten())
            .collect();
        let all: Vec<&Invariant> = records.iter().collect();
        let camel = concat!("fn assumedT", "emperature() {}");
        assert_eq!(breaks(camel, &all).len(), 1, "a casing walks through");
        let inside = concat!("    self.assumed_", "temperature_kelvin = 293.15;");
        assert_eq!(breaks(inside, &all).len(), 1, "a longer name walks through");
    }

    // The refusal surface. A fixture here is a whole file rather than a line,
    // because what the check compares is a file's module documentation against
    // its own code, and a line on its own carries neither half.

    fn a_file(module_docs: &str, body: &str) -> String {
        format!("{module_docs}\n\nfn judge() {{\n    {body}\n}}\n#[cfg(test)]\nmod tests {{}}\n")
    }

    #[test]
    fn a_file_that_decides_a_refusal_and_says_so_is_in_the_surface() {
        let source = a_file(
            &format!("//! A check.\n//!\n//! {SURFACE}"),
            "return Judged::Refused(why);",
        );
        assert_eq!(names_itself(&source), (true, true));
    }

    #[test]
    fn a_file_that_decides_a_refusal_without_saying_so_is_refused() {
        let source = a_file("//! A check.", "return Judged::Refused(why);");
        assert_eq!(names_itself(&source), (false, true));
    }

    #[test]
    fn a_file_that_says_so_and_decides_nothing_is_refused() {
        let source = a_file(&format!("//! A crate.\n//! {SURFACE}"), "let a = 1;");
        assert_eq!(names_itself(&source), (true, false));
    }

    #[test]
    fn a_comment_about_a_refusal_does_not_put_a_file_in_the_surface() {
        // The near neighbour, and the one that decides whether this check can
        // be lived with. A file that argues about refusals in prose, which
        // several here do, must not join the surface for using the words.
        let source = a_file("//! A crate.", "// what would be Judged::Refused elsewhere");
        assert_eq!(names_itself(&source), (false, false));
    }

    #[test]
    fn a_refusal_in_the_suite_is_not_the_surface() {
        // A fixture is a refusal written down to be read. Reading the suite
        // would put every file holding one into the surface, and the surface
        // would then be mostly suite.
        let source = "//! A crate.\n#[cfg(test)]\nmod tests {\n    Judged::Refused(why);\n}\n";
        assert_eq!(names_itself(source), (false, false));
    }

    #[test]
    fn this_tree_names_a_surface_and_it_is_not_empty() {
        // Every fixture above passes over a tree whose surface is empty, and
        // an empty surface names nothing for a coverage bar or a mutation run
        // to be pointed at. This is the leg that says the check found some.
        //
        // The sources are walked rather than asked of git. The check itself
        // asks git, because what is tracked is git's answer, but a test that
        // asked would fail in any copy of this tree that is not a checkout,
        // and a mutation run works in exactly such a copy.
        let crates = Path::new(env!("CARGO_MANIFEST_DIR")).join("..");
        let mut named = 0_usize;
        for entry in std::fs::read_dir(&crates).expect("the crates directory is in this tree") {
            let source = entry.expect("a readable entry").path().join("src");
            for file in ["lib.rs", "main.rs"] {
                let Ok(text) = std::fs::read_to_string(source.join(file)) else {
                    continue;
                };
                if names_itself(&text) == (true, true) {
                    named += 1;
                }
            }
        }
        assert!(named > 0, "nothing in this tree names itself as surface");
    }

    // A finding accepted rather than fixed. Every suppression below is
    // assembled from two halves, because this module sits under a subject the
    // check reads and a spelling written whole would be a suppression naming
    // no record, inside the fixtures for the check that refuses one.

    #[test]
    fn a_suppression_naming_a_record_that_is_in_the_tree_passes() {
        let records = ["docs/accepted-findings/a-lint-that-is-wrong-here.md"];
        let said = concat!(
            "#[all",
            "ow(clippy::x)] // docs/accepted-findings/a-lint-that-is-wrong-here.md"
        );
        let found = suppressions(said);
        assert_eq!(found.len(), 1, "{found:?}");
        assert_eq!(record_named(found[0].2, &records), Some(records[0]));
    }

    #[test]
    fn a_suppression_naming_no_record_is_refused() {
        let said = concat!("#[all", "ow(clippy::x)]");
        let found = suppressions(said);
        assert_eq!(found.len(), 1, "{found:?}");
        assert_eq!(
            record_named(
                found[0].2,
                &["docs/accepted-findings/a-lint-that-is-wrong-here.md"]
            ),
            None
        );
    }

    #[test]
    fn an_ordinary_attribute_is_not_a_suppression() {
        // The near neighbour. Attributes are most of what a Rust file carries,
        // and a check reading all of them as suppressions would refuse the
        // tree on the day it landed.
        for ordinary in ["#[derive(Debug)]", "#[must_use]", "#[test]", "#[cfg(test)]"] {
            assert!(suppressions(ordinary).is_empty(), "{ordinary}");
        }
    }

    #[test]
    fn the_workflow_auditor_is_silenced_by_a_comment_and_that_counts_too() {
        // The other tool this tree runs is silenced in a workflow file rather
        // than by an attribute, so the spelling is not a Rust one and the
        // subject is not only Rust files.
        let said = concat!("      # zizmor:", " ignore[unpinned-uses]");
        assert_eq!(suppressions(said).len(), 1);
        assert!(is_searched_for_a_finding(".github/workflows/gate.yml"));
        assert!(!is_searched_for_a_finding(
            "docs/accepted-findings/README.md"
        ));
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

    // The checks, given a tree.
    //
    // Everything above this line is a pure function handed its bytes. Each
    // check itself takes a path and asks git what is tracked, so the only way
    // to give one a subject is to build a tree and index it. A mutation run
    // over this file measured what that costs: the counters behind the
    // accounting, the guards that choose between refusing and passing, and the
    // line numbers a refusal prints were all reachable only by a real run, so a
    // negated counter or an inverted guard left the suite green.
    //
    // The counts are compared whole rather than searched for. A substring
    // assertion cannot tell a count from its negation, which is how the tail
    // was covered and still passed with `ran` counting downwards.

    /// A tree on disk with a git index, removed when it goes out of scope.
    ///
    /// This and the three helpers below are reachable from the suites of the
    /// other modules in this crate, because every check here asks git what is
    /// tracked and so every one of them needs a tree. A second builder written
    /// beside this one would be a second answer to what a tree is, and the two
    /// would agree until the day a check started reading something new.
    pub(crate) struct Tree(PathBuf);

    impl Tree {
        pub(crate) fn at(&self) -> &Path {
            &self.0
        }
    }

    impl Drop for Tree {
        fn drop(&mut self) {
            let _ = std::fs::remove_dir_all(&self.0);
        }
    }

    /// A directory of this process's own, named for the test that asked.
    ///
    /// The tests run in parallel in one process, so the name separates them
    /// from each other and the process id separates one run from the next.
    pub(crate) fn somewhere(named: &str) -> PathBuf {
        let at = std::env::temp_dir().join(format!("stoffbuch-{}-{named}", std::process::id()));
        let _ = std::fs::remove_dir_all(&at);
        std::fs::create_dir_all(&at).expect("a directory under the temporary directory");
        at
    }

    /// Builds a tree of `files` and indexes it, so `tracked` has an answer.
    pub(crate) fn a_tree(named: &str, files: &[(&str, &[u8])]) -> Tree {
        let at = somewhere(named);
        for (name, bytes) in files {
            let path = at.join(name);
            std::fs::create_dir_all(path.parent().expect("a file in a tree has a parent"))
                .expect("a directory in the temporary tree");
            std::fs::write(&path, bytes).expect("a file in the temporary tree");
        }
        git(&at, &["init", "--quiet"]);
        // The working copy is what the line-ending check reads, so nothing on
        // the way into the index may rewrite a byte of it. Left to the clone's
        // configuration, a carriage return fixture is normalised away on the
        // machine where the guard matters most.
        git(
            &at,
            &[
                "-c",
                "core.autocrlf=false",
                "-c",
                "core.safecrlf=false",
                "add",
                "--all",
            ],
        );
        Tree(at)
    }

    fn git(at: &Path, arguments: &[&str]) {
        let done = Command::new("git")
            .args(arguments)
            .current_dir(at)
            .output()
            .expect("git is on the path, since every check here asks it what is tracked");
        assert!(
            done.status.success(),
            "git {arguments:?}: {}",
            String::from_utf8_lossy(&done.stderr)
        );
    }

    /// What a check said when it refused nothing.
    pub(crate) fn note(judged: Judged) -> String {
        match judged {
            Judged::Nothing(Some(note)) => note,
            other => panic!("expected a clean verdict carrying a note, and got {other:?}"),
        }
    }

    /// What a check wrote when it refused.
    pub(crate) fn refusal(judged: Judged) -> String {
        match judged {
            Judged::Refused(why) => why,
            other => panic!("expected a refusal, and got {other:?}"),
        }
    }

    #[test]
    fn a_clean_tree_is_reported_with_the_count_of_what_was_read_and_nothing_else() {
        let tree = a_tree(
            "line-endings-clean",
            &[("a.md", b"one\n"), ("b/c.md", b"two\n")],
        );
        assert_eq!(note(no_carriage_return(tree.at())), "2 text file(s) read");
    }

    #[test]
    fn a_file_with_a_zero_byte_is_counted_as_skipped_rather_than_read() {
        let tree = a_tree(
            "line-endings-binary",
            &[("a.md", b"one\n"), ("b.bin", b"\x00\x01\x02")],
        );
        assert_eq!(
            note(no_carriage_return(tree.at())),
            "1 text file(s) read, 1 skipped as binary and not judged"
        );
    }

    #[test]
    fn a_tracked_file_absent_from_the_working_copy_is_counted_rather_than_passed_over() {
        let tree = a_tree(
            "line-endings-absent",
            &[("a.md", b"one\n"), ("gone.md", b"two\n")],
        );
        std::fs::remove_file(tree.at().join("gone.md")).expect("a file this test just wrote");
        assert_eq!(
            note(no_carriage_return(tree.at())),
            "1 text file(s) read, 1 tracked but not in the working copy"
        );
    }

    #[test]
    fn a_carriage_return_in_the_working_copy_is_refused_and_the_file_is_named() {
        let tree = a_tree(
            "line-endings-carriage-return",
            &[("a.md", b"one\n"), ("b.md", b"two\r\nthree\n")],
        );
        let why = refusal(no_carriage_return(tree.at()));
        assert!(
            why.contains("1 tracked text file(s) carry a carriage return"),
            "{why}"
        );
        assert!(why.contains("  b.md"), "{why}");
    }

    #[test]
    fn a_directory_git_knows_nothing_about_is_not_reported_as_a_clean_tree() {
        // The near miss for the whole apparatus. `git ls-files` fails here, and
        // a check that read a failure as an empty list would report every
        // subject clean without having seen one of them. The temporary
        // directory is outside any checkout, which is what makes git fail
        // rather than answer about somebody else's tree.
        let at = somewhere("not-a-repository");
        let judged = no_carriage_return(&at);
        let _ = std::fs::remove_dir_all(&at);
        assert!(
            matches!(judged, Judged::CouldNotJudge(_)),
            "expected a run that could not judge, and got {judged:?}"
        );
    }

    #[test]
    fn the_test_surface_check_counts_the_rust_files_and_leaves_the_rest() {
        let tree = a_tree(
            "headless-count",
            &[
                ("a.rs", b"fn a() {}\n"),
                ("b/c.rs", b"fn c() {}\n"),
                ("d.md", b"not rust\n"),
            ],
        );
        assert_eq!(
            note(a_test_reaches_nothing_it_may_not(tree.at())),
            "2 tracked Rust file(s) read, test code only"
        );
    }

    /// A tree carrying one held record, one unheld record, and a readme.
    ///
    /// The readme and the ordinary document are the two files that separate
    /// "a record under the records directory" from "any document anywhere". A
    /// check that read either as a record would refuse this tree, since neither
    /// carries the header a record is read for.
    fn an_invariant_tree(named: &str, extra: &[(&str, &[u8])]) -> Tree {
        let mut files: Vec<(&str, &[u8])> = vec![
            (
                "docs/invariants/README.md",
                b"# The records\n\nProse about the shape of one, and not a record.\n",
            ),
            (
                "docs/invariants/fixture-invariant.md",
                b"Id: fixture-invariant\nHeld: yes\nSubject: crates/\nSpelling: zzq-forbidden\n\
                  Retired-by: nothing, it is a fixture.\n\
                  Rule: this spelling may not appear under crates/ in the fixture tree.\n\n\
                  The body says what failure it prevents.\n",
            ),
            (
                "docs/invariants/fixture-unheld.md",
                b"Id: fixture-unheld\nHeld: no\nRetired-by: nothing, it is a fixture.\n\
                  Rule: a rule no search of the tree can hold.\n\n\
                  The body says why it cannot be held this way.\n",
            ),
            ("docs/elsewhere.md", b"An ordinary document.\n"),
            ("crates/clean.rs", b"fn a() {}\n"),
        ];
        files.extend_from_slice(extra);
        a_tree(named, &files)
    }

    #[test]
    fn a_tree_breaking_no_invariant_says_how_many_are_held_unheld_and_searched() {
        let tree = an_invariant_tree("invariants-clean", &[]);
        assert_eq!(
            note(no_invariant_is_broken(tree.at())),
            "1 held, 1 recorded as not holdable this way, 1 file(s) searched"
        );
    }

    #[test]
    fn a_broken_invariant_is_refused_at_the_line_it_is_on() {
        let tree = an_invariant_tree(
            "invariants-broken",
            &[(
                "crates/breaks.rs",
                b"fn a() {}\n\nfn b() { let x = ZZQ-FORBIDDEN; }\n",
            )],
        );
        let why = refusal(no_invariant_is_broken(tree.at()));
        assert!(why.contains("1 line(s) break an invariant"), "{why}");
        assert!(why.contains("crates/breaks.rs:3"), "{why}");
        assert!(why.contains("fixture-invariant"), "{why}");
        assert!(
            why.contains("this spelling may not appear under crates/"),
            "{why}"
        );
    }

    #[test]
    fn the_surface_is_the_files_that_name_themselves_and_the_rest_are_counted() {
        let judges = a_file(
            &format!("//! A check.\n//!\n//! {SURFACE}"),
            "return Judged::Refused(why);",
        );
        let quiet = a_file("//! Not a check.", "return 1;");
        let tree = a_tree(
            "surface-count",
            &[
                ("crates/judge.rs", judges.as_bytes()),
                ("crates/quiet.rs", quiet.as_bytes()),
                ("docs/note.md", b"not rust\n"),
            ],
        );
        assert_eq!(
            note(the_refusal_surface_is_named(tree.at())),
            "2 tracked Rust file(s) read, 1 in the refusal surface: crates/judge.rs"
        );
    }

    #[test]
    fn the_command_that_prints_the_surface_prints_the_same_files_the_check_placed() {
        let judges = a_file(
            &format!("//! A check.\n//!\n//! {SURFACE}"),
            "return Judged::Refused(why);",
        );
        let quiet = a_file("//! Not a check.", "return 1;");
        let tree = a_tree(
            "surface-printed",
            &[
                ("crates/judge.rs", judges.as_bytes()),
                ("crates/quiet.rs", quiet.as_bytes()),
            ],
        );
        assert_eq!(
            refusal_surface(tree.at()).expect("a tree git can be asked about"),
            vec!["crates/judge.rs".to_owned()],
            "the scope a mutation run is given is what the check placed, or the two drift"
        );
    }

    /// The spelling that turns a finding off, on a line that names a record.
    ///
    /// Written in halves for the same reason `SUPPRESSIONS` is: this file is
    /// under a subject the check reads, and a whole one here would be a
    /// suppression naming no record, sitting inside the check that refuses one.
    const NAMES_A_RECORD: &str = concat!(
        "    #[all",
        "ow(clippy::needless_range_loop)] // docs/accepted-findings/why.md"
    );

    /// The same spelling with nothing beside it.
    const NAMES_NOTHING: &str = concat!("    #[all", "ow(clippy::needless_range_loop)]");

    #[test]
    fn a_suppression_and_the_record_it_names_are_counted_and_nothing_is_refused() {
        let thing = format!("fn a() {{}}\n\n{NAMES_A_RECORD}\nfn b() {{}}\n");
        let tree = a_tree(
            "findings-clean",
            &[
                (
                    "docs/accepted-findings/README.md",
                    b"# Accepted findings\n\nProse, and not a record.\n",
                ),
                (
                    "docs/accepted-findings/why.md",
                    b"The finding, and the argument for accepting it.\n",
                ),
                ("crates/thing.rs", thing.as_bytes()),
                (".github/workflows/w.yml", b"name: w\n"),
                ("docs/note.md", b"outside the subject\n"),
            ],
        );
        assert_eq!(
            note(every_accepted_finding_is_recorded(tree.at())),
            "2 file(s) searched, 1 suppression(s), 1 record(s)"
        );
    }

    #[test]
    fn a_suppression_naming_no_record_is_refused_at_the_line_it_is_on() {
        let thing = format!("fn a() {{}}\n\n{NAMES_NOTHING}\nfn b() {{}}\n");
        let tree = a_tree(
            "findings-unrecorded",
            &[("crates/thing.rs", thing.as_bytes())],
        );
        let why = refusal(every_accepted_finding_is_recorded(tree.at()));
        assert!(
            why.contains("1 finding(s) turned off with the reason nowhere"),
            "{why}"
        );
        assert!(why.contains("crates/thing.rs:3"), "{why}");
    }

    #[test]
    fn a_record_no_suppression_names_any_more_is_refused_from_the_other_side() {
        // The register describes the tree rather than its own history, so it
        // fails closed in both directions. A reason left behind for something
        // that is no longer there reads as an accepted finding and is not one.
        let tree = a_tree(
            "findings-dangling",
            &[
                (
                    "docs/accepted-findings/why.md",
                    b"The finding, and the argument for accepting it.\n",
                ),
                ("crates/thing.rs", b"fn a() {}\n"),
            ],
        );
        let why = refusal(every_accepted_finding_is_recorded(tree.at()));
        assert!(
            why.contains(
                "docs/accepted-findings/why.md accepts a finding that nothing suppresses any more."
            ),
            "{why}"
        );
    }

    #[test]
    fn every_verdict_starts_at_the_same_column_whatever_the_part_is_called() {
        // The padding is what makes the report a column a reader runs down
        // rather than a ragged list. A width of nought or one leaves each
        // verdict at its own part's name length, so the parts stop agreeing.
        let longest = PARTS
            .iter()
            .map(|part| part.name.len())
            .max()
            .expect("the gate has parts");
        let columns: Vec<Option<usize>> = PARTS
            .iter()
            .map(|part| line(&ran(part.name), width()).find("  ran"))
            .collect();
        assert!(
            columns.iter().all(|at| *at == Some(longest)),
            "the verdicts do not line up: {columns:?} against a longest name of {longest}"
        );
    }
}
