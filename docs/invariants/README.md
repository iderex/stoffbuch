# Invariants the tree holds by searching itself

A schema decides the shape of one file. It cannot say that a rule holds across
the whole tree, and several rules this project states are exactly that shape: a
thing that must appear nowhere, or must appear only where it is allowed. Those
rules are otherwise sentences somebody has to remember, and a sentence somebody
has to remember is the failure this repository was built against.

Each one is a file in this directory. The gate reads them and searches the tree
for what they forbid. Which invariants exist is not written in this file or in
any other, because a list in a document drifts against the thing it describes.
The run prints them:

    cargo run --quiet --locked -p stoffbuch-cli -- gate

## The shape of a record

Header lines first, at column zero, then a blank line, then the body.

    Id:          the file name without its extension, and the name a refusal prints
    Held:        yes if the gate searches for this, no if it cannot be held this way
    Subject:     a path prefix, tracked files under it are searched
    Spelling:    text that may not appear in the subject, one line per spelling
    Retired-by:  what would make this invariant unnecessary
    Rule:        the sentence a refusal prints, in words somebody can act on

A record with `Held: no` carries `Retired-by` and `Rule` and neither `Subject`
nor `Spelling`. It exists so that an invariant nobody can hold this way is
written down as unheld rather than approximated by a pattern that half works and
then trusted. The body says why it cannot be held and what holds it instead,
where anything does.

The body of a held record says what failure the invariant prevents. Nothing
reads the body except a person, and that is what it is for.

## Why the spellings are literal text

There is no regular expression engine here. The lock file holds the five crates
of this workspace and nothing outside it, so a pattern language would be a
dependency taken for one check, and the means question is answered against the
standpoint each time rather than by habit.

Literal text is weaker than a pattern and the weakness is the point of the
`Spelling` line being repeatable: an invariant is a list of the spellings
somebody would actually write, chosen from what the mistake looks like rather
than from what the rule sounds like. What that cannot catch is a spelling nobody
listed. An invariant is a floor, and it holds what has actually been written or
what somebody expects to be written next.

## Why a refusal prints the rule and not the spelling

The risk with a check of this kind runs the other way from the usual one. A
pattern that refuses honest work is a pattern a contributor cannot argue with,
so it gets weakened until it holds nothing. Two things answer that.

A refusal prints the file, the line and the `Rule` sentence, so somebody who has
never opened this directory can tell what they did wrong and what to do instead.
A spelling printed on its own reads as an arbitrary ban.

And every held invariant carries two fixtures in the suite: one that trips it,
and a near neighbour that differs by as little as the surface allows and must
pass. The near neighbour is what catches a spelling so broad it refuses ordinary
work, on the day it lands rather than on the day somebody hits it. The suite
compares the set of records against the set of fixtures, so an invariant added
without either fixture reddens the suite.

## Why no record may name this directory

A record carries the spelling it forbids, so a record is a file containing the
thing it exists to refuse. A search that read this directory would refuse the
records, which is the shape where a checker refuses its own documentation.

This is not handled by quietly skipping the directory. A record whose `Subject`
reaches these files is refused by the loader with that reason, so the exclusion
is a thing the gate states rather than a silence a later reader has to infer.
The residual is that nothing searches the records themselves, and a spelling
that has to be forbidden inside a record has no home here.

The same problem appears one step further out. The suite's fixtures are code
under a subject, so a fixture written as a literal would trip the invariant it
is a fixture for. Every fixture spelling in the suite is assembled from two
halves at compile time for that reason, and what reaches the check is the text
somebody would actually write.
