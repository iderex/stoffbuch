# Findings accepted rather than fixed

An analyser names something and the honest answer is sometimes that the code is
right. This is where that answer is written down, so that it survives the tool
that produced the finding.

The directory is empty of records today. It is here before the first one for the
reason the argument in
[decisions/static-analysis-and-the-refusal-surface.md](../decisions/static-analysis-and-the-refusal-surface.md)
gives: the moment a first finding is accepted is the moment somebody reaches for
a dismissal button in a tool's own interface, and a register that already exists
is the one that gets used instead.

## The shape of a record

One file per accepted finding, named for what was accepted rather than for the
lint that fired, because a rule identifier changes with the tool and the reason
does not. Header lines first, at column zero, then a blank line, then the body.

    Finding:     what the tool said, in its own words
    Tool:        what said it
    Accepted-on: YYYY-MM-DD
    Retired-by:  what would make this record unnecessary

The body says why the code is right and the finding is not, in enough detail
that somebody who has never seen the line can agree or disagree with it. A
record whose body says the finding was noisy is a record that will be read as a
reason years after everyone who found it noisy has gone.

## The rule

A suppression names its record on the same line:

    #[allow(clippy::some_lint)] // docs/accepted-findings/<name>.md

The gate refuses both directions. A suppression that names no record here is
refused, because that is a finding turned off with the reason nowhere. A record
that no suppression names is refused as well, because a register that keeps
records for suppressions that no longer exist stops describing the tree and
starts describing its own history.

The subject is tracked files under `crates/` and `.github/`. This directory is
not searched, and neither is the rest of `docs/`, because a record carries the
spelling it is about and a search here would refuse the records themselves. The
residual is that a suppression written into a document is invisible to this,
which is the same residual the invariant records carry and for the same reason.

## What this is not

It is not a place to record a finding that was fixed. A fix is in the history
and needs no register.

It is not a way to lower a lint level for the whole tree. The levels are in the
workspace manifest and a change to them is a change to that file, argued there.
This register is for one line at a time, and a register with many records
naming one lint is the tree saying the level is wrong rather than the lines.
