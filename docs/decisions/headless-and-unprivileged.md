# A test may not need a display, rights, a fixed path or a network

Every test this project runs has to run on a machine with no display attached,
as a user with no administrative rights, with no network, and with nothing on
disk except what the test was given. That is a requirement of the first day
rather than something to find out on the day the suite stops running somewhere.

The requirement is not decorative here, and two routes into breaking it are
already visible in the plan. This project renders figures, because a comparison
of competing values is a thing people look at rather than read, and figure
rendering is the ordinary way a display dependency arrives. It also reads and
writes files constantly, which is the ordinary way a fixed absolute path arrives,
and a fixed path outside a temporary directory is how a test starts needing
rights it should not have.

## The rule

A test in this suite may not:

- open a window, or draw through anything that would open one
- require a display server, or read a display environment variable to decide
  what to do
- require administrative rights, elevation, or any prompt that asks for them
- register or start a service, a scheduled task or anything that outlives the
  test process
- read or write a fixed path outside the temporary directory it was given, and
  that includes a path under the user's home directory
- require a name to resolve, a socket to connect, or any other use of a network

Each of those is written as a thing a test may not do rather than as advice,
because advice is what a suite has when nobody can say whether it is being
followed.

Anything that genuinely needs one of them is not a test in this suite. It belongs
to the separate harness for what needs a network or a long run, and moving it
there is the repair rather than an exception to the rule.

## Figure rendering

A figure is produced by writing bytes to a file through a path that never opens a
window. Where a rendering library offers both an interactive backend and a file
backend, only the file backend is used, and the choice is explicit in the code
rather than inherited from whatever the environment happens to make the default.
An environment-chosen default is the same defect as an interactive backend: it
passes here and fails on the machine with no display, which is the failure this
rule exists against.

A figure's test asserts on the written bytes, or on properties of them that can
be measured, and never on anything a person has to look at. A test that renders a
figure and checks nothing is a test that the process did not crash, and it should
say that is what it is.

## What a check can refuse, and what it cannot

Part of this is refusable by reading the source. A literal absolute path, a
reference to a display environment variable, and a dependency on a library whose
whole purpose is a window are all things a check can name and refuse, and so is
a test that reaches the network through the obvious surfaces.

Part of it is not. A check reading source cannot tell that a library will open a
window three calls down, cannot follow a path that was assembled at run time, and
cannot see a name resolved by a dependency. That half is answered by running the
suite in an environment that has none of those things, which is a demonstration
rather than a check, and it is only as good as the environment it was run in.

Both halves are needed and neither substitutes for the other. The check catches
the ordinary mistake early and cheaply. The environment catches the mistake the
check cannot see, and only after somebody made it.

## What is refused today, and what is not

The gate carries a part named `headless` that reads the test code in every
tracked Rust file and refuses a spelling a test may not reach. What it covers is
printed by the run rather than listed here:

    cargo run --quiet --locked -p stoffbuch-cli -- gate

It refuses three of the six lines of the rule, and only where the surface is
written in the ordinary spelling. Reading a display environment variable.
Reaching the network through a socket type or a name resolution in the standard
library. Asking for administrative rights, elevation, a service or a scheduled
task by the name of the thing that grants one.

Three lines are not refused by anything, and a green run says nothing about
them. Opening a window has no spelling to match, because a rendering library
opens one through a name that is about drawing rather than about a window.
Reading or writing a fixed path outside the given temporary directory has no
arm in the check at all. Requiring a name to resolve inside a dependency is
invisible for the same reason a window is.

The check reads text a line at a time and carries no parser, so three more ways
past it are open. A spelling brought in under another name walks through. So
does a surface reached through a dependency that wraps it. So does test code in
a file that puts a test module above production code, because what is read is
everything from the first `#[cfg(test)]` line to the end of the file, plus the
whole of any file under a `tests/` directory, and that is a convention of this
repository rather than a fact the check establishes.

So the source half of this rule is partly held and the rest of it is prose. The
environment half is not held at all yet: nothing runs the suite anywhere except
on the machine of whoever typed the command, so neither the display
demonstration nor the unprivileged one has a place to happen. That is the state,
and it is written here rather than left for a reader to infer from a green run.

## What this record does not decide

It does not decide the harness for what genuinely needs a network or a long run,
which is its own work. It does not decide the rendering library or the file
format a figure is written in. It does not decide how a test is given its
temporary directory, nor the fixture convention a test uses to state the exact
bytes it is about. It does not decide which of the refusals above is implemented
first, or whether they are one check or several.
