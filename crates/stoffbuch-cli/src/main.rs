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
const USAGE: &str = "usage: stoffbuch gate

gate  run every part of the gate over the workspace this command was run in,
      in order, stopping at the first failure, and say what was examined

exit  0 nothing was refused, 1 something was refused, 2 could not judge
";

fn main() -> ExitCode {
    let given: Vec<String> = std::env::args().skip(1).collect();
    let given: Vec<&str> = given.iter().map(String::as_str).collect();

    let verdict = if given.as_slice() == ["gate"] {
        gate()
    } else {
        // Arguments the command does not understand are `2` rather than `1`: a
        // caller that reads a broken invocation as a clean refusal is wrong in
        // the direction that stays quiet.
        let _ = write!(std::io::stderr(), "{USAGE}");
        Exit::CouldNotJudge
    };
    ExitCode::from(verdict.code())
}

/// Runs the gate over the workspace the command was run in.
fn gate() -> Exit {
    let Ok(here) = std::env::current_dir() else {
        let _ = writeln!(std::io::stderr(), "the working directory could not be read");
        return Exit::CouldNotJudge;
    };
    let Some(root) = stoffbuch_gate::workspace_root(&here) else {
        let _ = writeln!(
            std::io::stderr(),
            "no workspace at or above {}, so there is no tree to judge",
            here.display()
        );
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
