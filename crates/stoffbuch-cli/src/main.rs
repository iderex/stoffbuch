//! The command line.
//!
//! The only part of this workspace with an audience that is a person, and so
//! the only one allowed to know about a terminal, an argument list or an exit
//! code. Everything it does it does by calling one of the other crates, so a
//! behaviour that exists only here is a behaviour no other consumer can reach.

use std::io::Write;
use std::process::ExitCode;

use stoffbuch_gate::Exit;

/// What the command understands, printed when it is given something else.
const USAGE: &str = "usage: stoffbuch <gate|surface>

gate     run every part of the gate over the workspace this command was run in,
         in order, stopping at the first failure, and say what was examined

surface  print the tracked files that decide refusals, one per line, so that a
         coverage bar or a mutation run reads the surface rather than a list

exit     0 nothing was refused, 1 something was refused, 2 could not judge
";

fn main() -> ExitCode {
    let given: Vec<String> = std::env::args().skip(1).collect();
    let given: Vec<&str> = given.iter().map(String::as_str).collect();

    let verdict = if given.as_slice() == ["gate"] {
        gate()
    } else if given.as_slice() == ["surface"] {
        surface()
    } else {
        // Arguments the command does not understand are `2` rather than `1`: a
        // caller that reads a broken invocation as a clean refusal is wrong in
        // the direction that stays quiet.
        let _ = write!(std::io::stderr(), "{USAGE}");
        Exit::CouldNotJudge
    };
    ExitCode::from(verdict.code())
}

/// The workspace the command was run in, or the reason there is none.
fn root() -> Option<std::path::PathBuf> {
    let Ok(here) = std::env::current_dir() else {
        let _ = writeln!(std::io::stderr(), "the working directory could not be read");
        return None;
    };
    let found = stoffbuch_gate::workspace_root(&here);
    if found.is_none() {
        let _ = writeln!(
            std::io::stderr(),
            "no workspace at or above {}, so there is no tree to judge",
            here.display()
        );
    }
    found
}

/// Prints the files that decide refusals, one per line.
///
/// One per line and nothing else, because what reads this is a shell loop
/// building the arguments of another run. A heading or a count here would end
/// up as a file name that does not exist, and the run pointed at it would say
/// it found nothing.
fn surface() -> Exit {
    let Some(root) = root() else {
        return Exit::CouldNotJudge;
    };
    match stoffbuch_gate::refusal_surface(&root) {
        Ok(surface) => {
            let mut out = std::io::stdout();
            for name in &surface {
                if writeln!(out, "{name}").is_err() {
                    return Exit::CouldNotJudge;
                }
            }
            Exit::Clean
        }
        Err(why) => {
            let _ = writeln!(std::io::stderr(), "the surface could not be read: {why}");
            Exit::CouldNotJudge
        }
    }
}

/// Runs the gate over the workspace the command was run in.
fn gate() -> Exit {
    let Some(root) = root() else {
        return Exit::CouldNotJudge;
    };

    let mut out = std::io::stdout();
    match stoffbuch_gate::run(&root, &mut out) {
        Ok(verdict) => verdict,
        Err(why) => {
            let _ = writeln!(std::io::stderr(), "the report could not be written: {why}");
            Exit::CouldNotJudge
        }
    }
}
