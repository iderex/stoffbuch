# Mutation testing over the surface that decides refusals

A suite that passes says the tests pass. It does not say the tests would notice
if the code were wrong. A mutation run makes one small change to the code at a
time, runs the suite over each, and reports the changes the suite accepted. Each
of those is a defect this tree would have shipped without anything going red.

## Where it points

At the files the gate names as the surface that decides refusals, and at nothing
else. A score over a whole tree goes up when somebody tests something easy, and
the code where a subtle error matters here is the code that decides whether a
record is refused: a comparison flipped in a dimension check, an off-by-one in a
locator parser, an inverted condition in an export minimum.

The scope is asked for rather than written down:

    cargo run --quiet --locked -p stoffbuch-cli -- surface

One file per line, from the same marker the gate part named `surface` reads and
refuses a disagreement about.
[decisions/static-analysis-and-the-refusal-surface.md](decisions/static-analysis-and-the-refusal-surface.md)
is where that marker is argued. A list of files in the workflow would be a
second home for a set that moves whenever a file starts or stops deciding
refusals, and a run pointed at the stale copy reports a clean score over code it
never touched.

## The first run, and what it found

Run against `main` at `4bae72a799683ce3832b50913b83fd26b57c7880`:

    cargo mutants --file crates/stoffbuch-gate/src/lib.rs -j 4
    136 mutants tested in 2m: 41 missed, 22 caught, 73 unviable

Forty-one survivors, and they were six shapes rather than forty-one questions.
Twenty-two were the counters behind the accounting, the files read, the files
skipped as binary, the files searched and the five totals in the closing line. A
mutant there leaves the verdict right and the disclosure wrong, and on this gate
the disclosure is the property the whole report rests on. Seven were the
emptiness guards that choose between refusing and passing, where deleting one
negation refuses a clean tree and passes a broken one. Four were in the function
that asks git what is tracked, where a mutant makes every check read a shorter
list and stay green. Two were the filters deciding which files are read as
records, two were the line numbers a refusal prints, and two were the column the
report pads its verdicts to.

They had one cause between them. Every check takes a path and reads the tree
through git, and no fixture gave one a tree, so what the suite exercised was the
pure half that compares text and never the half that decides.

One of them was a defect in a test rather than a gap in coverage. The tail's
counters could be negated and the suite stayed green, because the count was left
to be inferred and so was signed, and the assertion was a substring test: the
line read `-1 ran` and `contains("1 ran")` was satisfied by it.

## The triage

Every one of the forty-one became a test. The checks are now reached with a tree
in hand, so the counters, the guards, the tracked-file walk, the record filters
and the printed line numbers are all executed by a proof rather than argued
about. Counts are compared whole rather than searched for, since a substring
assertion cannot tell a count from its negation, and the tail's counters are
unsigned, so the same mutation stops the run instead of printing a plausible
line.

Nothing was recorded as not mattering. The two column-width mutants were the
candidates, and a test asserting that every verdict starts at the same column
was cheaper than the argument for leaving them alive.

## The threshold, and the date it was set

The same run again, over the same file, with the tests above in place:

    cargo mutants --file crates/stoffbuch-gate/src/lib.rs -j 4
    140 mutants tested in 26m: 63 caught, 77 unviable

Zero survivors, and the run takes thirteen times as long as it did, because
almost every mutant now has a suite that builds a tree and reads it rather than
one that compares two strings.

No mutant survives, and that is the threshold: a surviving mutant reds the run.
Set on 2026-08-09 from the measurement above rather than chosen in advance,
because a number picked before the survivors are triaged is a guess that later
gets quoted as a standard.

It is a threshold that will need revisiting rather than one that holds forever.
It is affordable while the surface is small, and the honest moment to change it
is when a run over a wider surface makes it a wall rather than a bar. Whoever
changes it states the measurement it was changed from and the date, in this
section, the same way this one does.

## The threshold is not met today

The triage above is one file, and the surface names two. The second was added
the day this was measured, and a run over it was started and stopped after seven
of its one hundred and sixty-six mutants. One of the seven survived:

    cargo mutants --file crates/stoffbuch-gate/src/canonical.rs -j 4
    crates/stoffbuch-gate/src/canonical.rs:97:16: replace < with > in canonical

So there is a comparison on the surface that can be reversed with nothing going
red, the rest of that file is unmeasured, and the first scheduled run over the
whole surface should be expected to refuse. That is a true statement about the
tree rather than a check that arrived broken, and the alternative, a threshold
written to fit what happens to pass today, is the thing that makes a green run
mean nothing.

[#114](https://github.com/iderex/stoffbuch/issues/114) holds the full run over
that file and the triage of what it finds.

## Why it is off the pull request path

A run costs tens of minutes over a gate that finishes in well under a minute, so
it goes on a schedule and never on a change.

`.github/workflows/mutation.yml` is the run, weekly and on request, with no
pull-request trigger.

## What this measurement is not

It was made on one machine, and the scheduled run happens on a Linux runner. The
suite is the same suite and the mutants are generated from the same source, so
the numbers should agree, but they have not been compared and the first
scheduled run is its own measurement rather than a confirmation of this one.

The unviable mutants are not a result. They are changes that did not compile,
and a change that does not compile says nothing about the suite.

The count moves with the version of the runner, because the set of mutants it
generates is part of that tool rather than part of this tree. The workflow pins
the version for that reason, and a run under a different one is a different
measurement wearing the same number.
