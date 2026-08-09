# How a test states the exact bytes it is about, and what a fixture may be about

The checks in this project refuse files. A test of such a check has to present
the exact bytes that should be refused, and exactness here is harder than it
looks, because a file on its way into version control passes through
normalisation that can change precisely the bytes a check exists to catch.

Two rules follow, and neither is worth much unless it is written down before
there are many checks to retrofit it across. The first is where a fixture's
bytes live. The second is what a fixture may be about.

## Bytes a checkout would change are written in the source of the test

`.gitattributes` declares `text=auto eol=lf` over everything in this tree and
repeats it by name for the file types whose bytes a digest or a canonical form
is a claim about. So a carriage return cannot enter the tree and cannot come
back out of a clean one. That is the guard working, and it is also why a fixture
whose whole point is a carriage return cannot be a file: it would be written as
line feeds on the way in, the test would pass, and it would prove nothing about
the check.

Such a fixture is therefore a byte string in the source of the test, with the
endings written as escapes. What reaches the check is then exactly what the test
wrote, on every clone, whatever that clone's line ending configuration says.

The other direction closes it. A carriage return that did reach a working copy
is refused by the gate, which reads the working copy rather than the blob, so a
fixture file that depended on one could not sit in the tree either. Between the
declaration and that refusal, a fixture that relies on a carriage return
surviving version control is not something a contributor can write here.

The same reasoning reaches any check whose subject is a byte the tooling around
the file might touch. Where it does, the fixture goes in the source. Where a
fixture is ordinary text whose exact bytes nothing normalises, a file is fine,
and the schema fixtures under `crates/stoffbuch-schema/fixtures/` are that case:
they are whole rows, they are read as rows, and what would break them is a
change to their content rather than to their line endings.

### What this leaves, named rather than left to be found

Only the line endings are converted. `.gitattributes` declares no working tree
encoding, so a fixture's encoding is its own, and git does not strip trailing
whitespace on the way in either. A fixture may carry both and they survive.

Nothing in this tree is marked binary, so nothing sits outside the declaration
today. Whoever adds the first such pattern is taking that file type out of the
reach of both halves above, and the reason for it belongs beside the pattern.

And a spelling written whole in a test can be the thing the test is about. Two
checks here read the suite itself, so a fixture for them written as a plain
literal would make the suite refuse itself. Those fixtures are assembled from
halves that are joined before anything runs, so the check is given the spelling
a contributor would actually write, and the file the check reads does not carry
it. The alternative is an exception list, which is the shape that grows until
the check means nothing.

## A fixture is invented material, never the register

Fixtures use invented materials, invented sources and invented quantities, and
they stay that way.

A test that ran a check against the real register would prove the state of the
register on the day it ran and not that the check works. When the register
changes, such a test either breaks for a reason that has nothing to do with the
change under review, or, worse, keeps passing because the case it was about has
been curated away. The second failure is silent and is the one that matters: a
green suite would then be reporting on data rather than on a guard.

Nothing under `crates/` reads the register. The one place the word appears is
the sentence the gate prints about what it examines:

    git grep -n 'register/' -- crates/
    crates/stoffbuch-gate/src/lib.rs:165:        examines: "every file under register/, against the schema for its row kind",

read at `8cb40c9`, when the register held no files at all. The rule is written
now rather than when it first has something to refuse, because the first test
that reaches for a real row will be written by somebody who wanted one example
and had one to hand.

### Reading the tree is not reading the register

Several tests here do read the real tree, and that is a different thing. The
suite reads the invariant records to check that every held invariant has its
fixtures, reads the schema directory to check that every schema is one the suite
knows about, and reads the crate sources to check that the refusal surface this
tree names is not empty. Each of those is an accounting question about a set of
files, and the answer is meant to move when the set moves. None of them decides
whether a check refuses the right bytes; that is always decided against a
fixture.

The register is `register/`, and the reason it is the boundary is in
`tree-layout-and-file-naming.md`: what is under it is the published product and
what is beside it is not.

## A fixture is a near miss

A fixture that could not plausibly have been written by mistake proves less than
one that nearly is correct. A check given nonsense proves that it rejects
nonsense, and nobody was going to write nonsense.

So a tripping fixture is the one-character mistake somebody will actually make,
and beside it goes a near neighbour that differs by as little as the surface
allows and passes. The neighbour is what catches a check so broad it refuses
honest work, on the day it lands rather than on the day a contributor hits it
and starts weakening it. `CONTRIBUTING.md` carries the rule with worked examples
for somebody writing their first one.

Two further obligations follow from what a pair of fixtures can and cannot show.

Where a check can refuse for more than one reason, the test asserts which reason
it refused for rather than that it refused. A check that started refusing
everything passes a test that only asked whether something was refused, and that
is the failure a pair of fixtures does not catch on its own.

Where the set of things needing fixtures is derived from something in the tree,
the suite compares the two sets rather than trusting a list. An invariant added
with no fixtures reddens the suite, and so does a fixture left behind by a
record that was deleted. Which checks exist, and what each one examines, is
printed by the run rather than listed here:

    cargo run --quiet --locked -p stoffbuch-cli -- gate

## What this record does not decide

It does not decide the fixture format for any particular check, which belongs
with that check. It does not decide where fixtures sit in the tree beyond that
they are not under `register/`; the layout record settles the rest and states
that test fixtures are outside what it decides. It does not decide what the
conformance suite holds or what shape its cases take, which is the format's own
work and is data rather than a test in any language. It does not decide anything
about the review, which is where a fixture that is technically a near neighbour
and practically meaningless is caught.
