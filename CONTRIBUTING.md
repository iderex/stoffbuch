# Contributing

The work here is curation. Most of what this register needs is a person who has
a paper in front of them and can say what it actually reported, under what
conditions, and how well it was known. This document is written for that person,
and it takes you from a source in hand to a row you can propose, without asking
you to work out the conventions from files that are already here.

## Run the gate before you send anything

    cargo run --quiet --locked -p stoffbuch-cli -- gate

That is the whole thing. It runs every part in order, stops at the first
failure, and prints what it examined and what it did not. No part of it is
listed in this document, because a list written here drifts against the run and
the run is the authority. Read what it printed rather than assuming what it
covers.

There is a hook that runs the same command before a push:

    git config core.hooksPath .githooks

It is a convenience that shortens the wait, and it is not the enforcement. A
clone that never ran that line does not have it, `--no-verify` skips it, and
nothing on the server knows either way. What stands behind a merge is the run on
the server and the review.

## What a contribution usually is

A row, and a source if the paper you cited is not already in the register. A
test only when a check changed. Most contributions here change no code at all.

Every change starts as an issue and lands as a pull request. An issue says what
is wrong, what the evidence is, and what done means. If the evidence is a
number, it carries the command that produced it.

## From a source in hand to a proposed row

**Is the quantity one this register holds.** The boundary is decided in
`docs/decisions/out-of-scope.md`, including the test a proposed quantity has to
pass and what happens to a request that falls outside. Read it there rather than
here; it is argued at a length this document should not repeat.

**Decide which kind of row you are writing.** A number a publication measured
and reported, a coefficient set fitted to data, and a number this project
computed are three different claims and three different kinds.
`docs/decisions/record-kinds.md` says what each kind may and may not assert.
Only the measured value kind has a schema in the tree today.

**Mint an identifier.** It is opaque and random, minted once, never reused, and
never derived from anything about the row, so that nothing the register later
learns about the sample can make the identifier wrong. The exact form is in the
schema. Pick the characters at random rather than from anything meaningful, and
check that the tree does not already carry it.

**Put the file where it goes.** A row at a version lives at

    register/rows/<identifier>/<identifier>@1.json

with the version in the file name and in the file. A correction later adds a
file beside it rather than editing this one. `docs/decisions/tree-layout-and-file-naming.md`
has the whole layout, including where a source, a form and a large tabulated
block sit.

**Fill in the row against `schema/measured-value.schema.json`.** That file is
the specification. Every field carries, beside it, what it means and why it
exists, and there is deliberately no second document describing the fields in
prose, because two homes for one fact drift and the prose is the one that
drifts. Open it and work down it.

Two things you will reach in it that the tree cannot yet resolve. A row names a
quantity from the quantity register, and a chain link names a source from the
source register, and neither register is in the tree. Until they are, write the
identifier you think is right and say in the pull request that you chose it;
settling it is part of the review rather than something you can look up.

**Write it in the canonical form.** The rule is in
`docs/decisions/means-format-and-language.md`, in full and with the reason a
formatter that reparses a number destroys a claim the register exists to carry.
The parts of it you will feel while writing by hand are the two spaces per level
of nesting, the object keys sorted by their bytes, and the number written
exactly as its digits stood. No formatter exists yet, so the form is currently
held by a person reading the file.

**Run the gate, then send it.** Nothing in the gate validates a row against the
schema yet. That check is issue #32 and until it lands a well formed row rests
on you and on the review, so a run that went green has said nothing about your
row.

## What you have to have read

Say which link in the chain you read, and say it honestly. A curator who read a
handbook and copied its citation of a paper from 1974 has not read the paper
from 1974, and the row says so: the chain carries every link the curator knows
about, and exactly one of them is marked as the one they read.

That marking is what makes a value taken from a compilation rather than from the
original visible. Such a row is written with the extraction that says it was
copied from a compilation, and its chain has the compilation first and the
original behind it. A row that quietly presented a compilation's number as
though the original had been read is the defect this register exists to remove,
and it is not caught by any check here.

`docs/decisions/provenance-and-the-citation-chain.md` argues the model, and
states plainly the one thing it cannot check: whether the number really is on
that page of that paper. That is what the review is for.

## What is not accepted

`docs/decisions/out-of-scope.md` is the boundary and this document does not
restate it. Read it before proposing a quantity, and read what it says about a
request that falls outside, because the answer is not always no.

## Fixtures, and the near miss

A test of a check has to present the exact bytes the check should refuse, and
some of those bytes do not survive being a file in the tree. How a test states
them, and why a fixture uses invented material rather than the real register, is
in `docs/decisions/fixtures-and-what-a-test-may-be-about.md`.

The part that matters most when you write one is what the fixture is. A fixture
that could not plausibly have been written by mistake proves less than one that
nearly is correct. Reach for the one character somebody will actually get wrong.

Two from this project's own subject matter.

A unit that is wrong and has the right dimension. A row reporting a band gap as
`3.42` with the unit `meV` where the publication printed `eV` is off by a factor
of a thousand, and both units parse, both are energies, and a check that asked
only whether the unit parses passes it. The fixture that proves such a check is
that row; its near neighbour is the same row with `eV` and nothing else changed.
A fixture using a unit that is not a unit at all would have proved only that the
parser rejects nonsense.

A trailing zero. An uncertainty printed as `0.020` and one printed as `0.02` are
different claims about the third figure, and they are the same number to
anything that parses them into a binary floating point value on the way past.
The fixture is the first written against the second, differing in one character,
which is what a formatter that reparsed the number would silently turn one into.

The mistake no check here catches is the transposed digit, `3.24` for `3.42`,
because nothing in this tree has read the paper. That one is the review's, and
saying so is better than leaving you to assume the gate covers it.

## Terms

The repository is under AGPL-3.0 and the text is in `LICENSE`. Contributions
from outside are accepted, under that licence and under a sign-off.

What is still open, and is issue #1 rather than something this document decides:
which of the two SPDX identifiers for this licence holds, whether per-file
notices are carried, and what licence the register rows themselves are under as
distinct from the code. If any of those matters to you before you contribute,
say so on that issue and get an answer rather than an assumption from here.

## Sending it

Sign off on your commits:

    git commit -s

That puts a `Signed-off-by` trailer on the commit matching its author, and a
pull request is refused without one on every non-merge commit. What the trailer
asserts is the text in [DCO](DCO) at the root of this tree, which is the
Developer Certificate of Origin at version 1.1. Read it before you sign
anything: the trailer is short and what it commits you to is not in it.

A commit message states what changed and what failure it prevents. Where you are
correcting something, it says what was wrong and how it was found. One topic per
commit and per pull request.

In the pull request body, put the commands you ran and what they printed. A
number without the command that produced it is a claim, and this project writes
claims as claims.
