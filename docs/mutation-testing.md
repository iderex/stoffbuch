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

## The second file, and what a full run over it found

The triage above is one file and the surface names two. The full run over the
second, on `main` at `591b79e2aba896a35a047e90ab0bdc1e45b37c29`:

    cargo mutants --no-shuffle --file crates/stoffbuch-gate/src/canonical.rs -j 4
    166 mutants tested in 37m: 17 missed, 112 caught, 32 unviable, 5 timeouts

Seventeen survivors, in six shapes.

Seven were the escape table in the string reader, one arm each for `\"`, `\\`,
`\b`, `\f`, `\n`, `\r` and `\t`. Delete an arm and that escape becomes a
spelling the grammar does not have, so a record carrying it is refused for being
correct, which is a refusal a curator cannot act on. The only two escapes the
suite reached were the two the form does something visible with: the solidus it
writes out, and the surrogate pair it joins.

Four were the counters in the check that reads the tree, the records read and
the records tracked but absent from the working copy. A mutant there leaves the
verdict right and the disclosure wrong, which is the same shape the first run
found in the same place and for the same reason.

Two more were in that function and are not counters. One is the negation that
chooses between refusing and passing: without it the check refuses a clean tree
and passes a broken one. The other is the comparison deciding whether the absent
files are mentioned at all, which is the difference between a count a reader can
act on and one that says a register was read whole when part of it was not
there.

Two were the step over the sign of a negative number. No fixture in the suite
had one, and a published coefficient is as often negative as not, so the reader
could stop taking half the numbers this register is for and nothing would say
so.

One was the arm refusing an unescaped control character inside a string, which
is how a tab pasted out of a table reaches a record.

One was the comparison deciding whether anything follows the value a file holds.
That is the survivor this was opened on and the reason it was known before the
run was made: reversed, a file carrying a second value after the first is
accepted, and the form written back is the first value alone with the rest gone
and nothing in a diff to say so.

Five mutants are reported as timeouts rather than as caught or missed. Each is a
step through the text replaced by one that does not advance, so the reader loops
and the suite never finishes. They are counted apart from the survivors, and no
test kills one: a test asserts something about what the code returned, and this
code returns nothing. What stops them is the run's own clock.
[#116](https://github.com/iderex/stoffbuch/issues/116) holds them and says what
they are a fact about.

## The triage of the second file

Every one of the seventeen became a test and nothing was recorded as not
mattering. The escapes are a row per escape rather than one row carrying all of
them, so a failure names the one the reader lost. The negative number and the
control character are fixtures of the shape a curator actually writes. The
counters, the negation and the comparison are reached the only way they can be,
by giving the check a tree with a register in it, which is what the first file's
triage had to do as well.

What says each of the seventeen is dead is the run over the whole surface below
rather than the argument above. That run applies every mutant on its own and
reports which the suite let through, so a test written to kill one and missing
it shows there rather than in a sentence here.

The tree builder the first file's suite carries is used rather than a second one
written beside it. Two builders would be two answers to what a tree is, and they
would agree until the day a check started reading something new.

## The threshold, and the date it was set

The first file again, with its own triage in place:

    cargo mutants --file crates/stoffbuch-gate/src/lib.rs -j 4
    140 mutants tested in 26m: 63 caught, 77 unviable

Zero survivors, and the run takes thirteen times as long as it did, because
almost every mutant now has a suite that builds a tree and reads it rather than
one that compares two strings.

No mutant survives, and that is the threshold: a surviving mutant reds the run.
Set on 2026-08-09 from the measurement above rather than chosen in advance,
because a number picked before the survivors are triaged is a guess that later
gets quoted as a standard.

The whole surface, with both triages in place:

    cargo mutants --no-shuffle --file crates/stoffbuch-gate/src/canonical.rs --file crates/stoffbuch-gate/src/lib.rs -j 8
    306 mutants tested in 33m: 192 caught, 109 unviable, 5 timeouts

No mutant survives the whole surface, measured on 2026-08-10. The job count is
higher than the one the workflow gives it because this machine has more
processors to spare; how many run at once moves the wall clock and not the set.

That run still exits non-zero, and what makes it non-zero is the five timeouts
rather than anything the suite let through:

    echo "exit=$?"
    exit=3

The run is red and nothing survived it, and those are two sentences about the
same run rather than one. Reading the first as the second is the mistake this
section exists to stop, which is why the threshold is written about survivors
and not about the exit code.

It is a threshold that will need revisiting rather than one that holds forever.
It is affordable while the surface is small, and the honest moment to change it
is when a run over a wider surface makes it a wall rather than a bar. Whoever
changes it states the measurement it was changed from and the date, in this
section, the same way this one does.

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
