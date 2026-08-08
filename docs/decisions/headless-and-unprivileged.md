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

Nothing refuses any of this today, because there is no code in the tree and no
check to put it in. Until that exists this record is a rule with nothing behind
it, and every route in this project passes a test that violates every line above.
That is the state, and it is written here rather than left for a reader to
discover from a green run.

## What this record does not decide

It does not decide the harness for what genuinely needs a network or a long run,
which is its own work. It does not decide the rendering library or the file
format a figure is written in. It does not decide how a test is given its
temporary directory, nor the fixture convention a test uses to state the exact
bytes it is about. It does not decide which of the refusals above is implemented
first, or whether they are one check or several.
