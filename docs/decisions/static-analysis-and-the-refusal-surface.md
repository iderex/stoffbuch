# Static analysis, and the surface that decides refusals

Analysis over the whole codebase is the ordinary half and it is the smaller one.
The half that matters here is naming the code that decides whether a record is
refused, because a check that fails open lets a wrong number into the register
and every result computed from that row inherits it. That surface is small, it
is identifiable, and holding it to a higher standard than the rest is the shape
this project wants.

## The tooling, and why this one

The analyser is the one the language ships with, run over every crate and every
test target as the gate part named `lint`. It was chosen because it fails a run
rather than filing a finding somewhere a person has to go and look, it needs no
account and no service that can be withdrawn, and it adds nothing to the lock
file. The choice on the board this repository takes its quality target from does
not transfer, because that choice is bound to a different language rather than
to a property.

The levels are declared once, for the whole workspace:

    git grep -n -A3 'workspace.lints.clippy' -- Cargo.toml
    Cargo.toml:29:[workspace.lints.clippy]
    Cargo.toml-30-all = { level = "deny", priority = -1 }
    Cargo.toml-31-pedantic = { level = "deny", priority = -1 }

Declaring them in the workspace manifest rather than per crate is what stops a
crate opting out by forgetting an attribute, and the negative priority leaves
room to set one lint differently later without reopening the group.

Denying the pedantic group is a real position with a real price, and the price
is worth writing down because it is what the first person to find it
inconvenient will argue with. That group holds lints that are judgements about
style rather than reports of a defect, some of them fire on code that is correct,
and every one of them stops a build. The reason to pay it is that this tree's
product is a register whose whole claim is that a number in it can be traced,
and the failure mode of a lint set to warn is a warning nobody reads sitting
above the line where a wrong number got in. A warning that stops nothing is a
warning that stops nothing on the day it matters.

The way out of a lint that is wrong for a particular line is not to lower the
group. It is to accept that finding, in the register below, where the reason
survives the tool.

## Naming the surface

The surface is named by a line in a file's own module documentation:

    Refusal surface.

A file carrying that line is part of the code that decides refusals. The gate
part named `surface` refuses both directions of a disagreement between what a
file says and what it does: a file that decides a refusal and does not carry the
line, and a file that carries the line and decides none. The first is a piece of
the surface that a coverage bar or a mutation run would never be pointed at. The
second widens the surface until it names most of the tree, and a standard that
covers everything is one nobody can meet.

The alternative was a list of modules in a record. It was refused because a list
somebody has to remember to update is stale by the third check, and the two
issues that consume this naming are checks three and four. A marker stays
attached to the thing it names, and moving a file moves it.

What consumes it does not read this document. The run prints the surface:

    cargo run --quiet --locked -p stoffbuch-cli -- gate

and the line under `surface` names the files, for a person reading the report.
What a run reads instead is

    cargo run --quiet --locked -p stoffbuch-cli -- surface

which prints one file per line and nothing else, because what consumes it is a
loop building the arguments of another command, and a heading or a count in that
output becomes a file name that does not exist. Both come from one walk over the
tracked files, so the surface a run is pointed at and the surface the check
placed cannot differ. Nothing here lists them, for the same reason nothing here
lists the gate's parts.

## What that naming cannot do

It is a file and not a function. A file holding both a check and the report it
writes puts the report inside the surface, and a coverage number over such a
file is measuring more than the surface. What fixes that is splitting the file
when it is worth splitting, and until then the number is generous rather than
wrong, which is the direction that does not hide a hole.

It reads text. A file that matches on a refusal, rather than deciding one, reads
the same as a file that decides one and is asked for the line as well. That
names more of the surface rather than less, so it is left as it is rather than
made clever.

It reads production code only, stopping at the first test module. A fixture is a
refusal written down to be read, not one a run acts on, and a suite that put
itself in the surface would make the surface mostly suite.

## A finding accepted rather than fixed

Some findings are not defects. A lint fires on a line that is right, an auditor
names a step that is deliberate, and the honest answer is to accept the finding
rather than to contort the code around it.

An accepted finding is recorded in the tree, in `docs/accepted-findings/`, and
the suppression that turns it off names the record on its own line. Both
directions are refused by the gate: a suppression naming no record, and a record
no suppression names.

The reason is that the alternative loses the record. A finding dismissed in a
tool's own interface lives in that tool's database, and the day the tool is
replaced, the suppression is gone with it and nobody can say why it was
acceptable. What a later reader needs is not the dismissal, it is the argument.

The register is empty today, and it is built now on purpose. The first accepted
finding is exactly the moment somebody reaches for the tool's own dismissal
button, and a register that exists by then is a register that gets used.
