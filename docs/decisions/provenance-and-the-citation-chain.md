# The provenance model and the citation chain

The complaint this project exists to answer is that nobody can trace which
measurement a given value came from. So the provenance model is the load-bearing
decision of the register, and everything in it is a field rather than a sentence.

The nearest existing collection shows the shape of the problem. Its reference is
free prose with a link inside it and everything about the sample is a comment:

    curl -s https://raw.githubusercontent.com/polyanskiy/refractiveindex.info-database/main/database/data/main/Si/nk/Aspnes.yml | head -12

Run it and the reference comes back as a sentence with an HTML anchor around a
digital identifier, and the crystal orientation, the doping and the temperature
come back inside a comment field, the temperature as the words "Room
temperature". That is a real citation and a real set of conditions, and neither
can be read by a machine, joined against another entry, or checked for resolving
to anything. There is no locator at all: the reference names the paper and the
reader is left to find the number in it.

## Four parts

### The source

The published thing. It lives once, in its own tree, and rows point at it, so a
citation cannot drift between two rows that mean the same paper.

A source record carries a type from a closed vocabulary, a structured citation,
and an identifier where one exists. The type is required because a journal
article, a conference paper, a technical report, a book or chapter, a standard
and a vendor document are found by different routes and a reader needs to know
which they are looking for.

The structured citation is required always. It is fields, never a sentence:
authors, title, the container and its volume, issue and pages where there is
one, the publisher and edition where it is a book, the issuing body and
designation where it is a standard or a report, and the year. A source with
neither an identifier nor a structured citation is refused.

The identifier is optional, because a large part of what this register is for
sits in papers, reports and handbooks from before digital identifiers existed.
It is a form with a name rather than a URL: a DOI, an arXiv identifier, an ISBN,
a report designation issued by the body that issued the report, a standard
designation with its year. The forms the register accepts are a closed set, and
each has a shape a machine can refuse.

### The locator

Where inside the source. This is the part existing collections omit, and without
it a citation names a paper and leaves the reader to find the number.

The locator belongs to the row, not to the source, because two rows cite one
paper at different tables. It is required on every row that cites a source. Its
vocabulary is decided where the source register is built; what is decided here is
that it is a vocabulary rather than prose, that it covers at minimum a table with
a row or column reference, a figure with a curve reference, a page and an
equation, and that a locator that does not parse is refused.

### The extraction

How the value got from the source into the register. A closed vocabulary, and
each entry owes something different.

`transcribed` means read off a printed table or a printed statement. It requires
a locator that resolves to a table with a row or column reference, or to a page
and an equation. It owes nothing further.

`digitised` means read off a published figure. It requires a locator naming the
figure and the curve. It owes a digitisation uncertainty, which is not optional
and is not zero, because a point read off a printed axis carries an error that
the publication's own uncertainty says nothing about. It also owes the method,
because a point read by eye off a scanned page and a point extracted by software
from a vector figure are not the same claim.

`copied-from-compilation` means taken from something that had itself taken it.
It owes a chain of at least two links, with the read link on the compilation
rather than on the original, which is the honest statement that the curator did
not open the original.

The fourth case the plan raised, a value the curator computed from other
quantities the same publication reported, is not an extraction method and is not
in this vocabulary. It is a derived value: the quantities the publication printed
become measured rows with their own locators, and the computed number names them
as inputs. See `record-kinds.md`. Putting it here instead would let a number this
project computed enter the register as though a publication had printed it,
which is the one thing the derived kind exists to prevent.

### The chain

The case this project is actually about. A value appears in a review, the review
credits a paper, the paper credits an earlier measurement.

A row carries the whole chain it knows about, as an ordered list of source
references running from where the curator got the value back to the earliest
source the curator could identify. A row with a single source is the ordinary
case and is a chain of length one, not a special shape.

Exactly one link carries the mark saying the curator read it. A curator who read
a handbook and copied its citation of a 1974 paper has not read the 1974 paper,
and the mark is what stops the register presenting them as though they had. A
chain with no mark and a chain with several are both refused.

A chain ends somewhere, and the register distinguishes two ways of ending. The
last link carries a terminating reason when the curator established that the
earliest source gives no citation, which is a fact about the field and a
complete statement. A last link with no such reason means nobody has finished
walking it, which is an open piece of work. An empty field would collapse the
two, and the collapsed version reads as the first while meaning the second.

## What a source identifier is when there is none, and what is still checkable

The identifier is absent and the structured citation carries the whole weight.
Nothing about the absence is written as an apology or a placeholder; a source
with no identifier is an ordinary source.

Without an identifier a check can still verify that the citation carries every
field its source type requires, that the year parses and is not in the future,
that the type is in the vocabulary, that no two source records in the tree are
identical in their citation fields, and that the source is referenced by at least
one row. What it cannot verify is that the source exists, which is the same thing
it cannot verify when an identifier is present and unresolvable offline.

## What a report says about a chain nobody has walked

The coverage report counts, for the whole register or any subset, how many rows
carry a read mark that is not on the earliest link of their chain. Those are the
rows resting on somebody else's reading, and that count is the honest measure of
how much of this register is a compilation of compilations.

It prints as a count with the command that lists the rows behind it, so the set
can be worked through rather than admired. The number is expected to be large
early and is not a defect. What would be a defect is the number not existing.

## What a machine can refuse

- a row citing no source at all, where the row is a measured value or a fitted
  model
- a source identifier that does not match the shape of the form it declares
- a row with no locator
- a locator that does not parse
- an extraction method outside the vocabulary
- a digitised row with no digitisation uncertainty
- a chain link naming a source that is not in the source register
- a chain carrying no read mark, or more than one
- a cycle in a chain
- a source in the tree that no row references, which is reported by the gate
  rather than left to accumulate

## The one thing this model cannot check

Whether the citation is correct. No reading of this tree can tell whether the
number in a row is the number printed at that locator in that source, whether
the chain links really cite one another, or whether the curator read what the
mark says they read.

That is caught in review, by a reader with the source in hand, and it is the
reason the review of a curation change is a reading of the source rather than a
reading of the diff. It is caught a second time, weakly and late, by the
disagreement report: a transposed digit usually shows up as a value that sits
apart from everything it should agree with. Neither route is a check, and neither
is described here as one.

## What this record does not decide

It does not decide whether a source record carries a statement of the terms the
curator understood to apply to it. That question is part of the reading of legal
terms that is reserved elsewhere, and this record neither assumes an answer nor
provides a field for one.

It does not decide whether values that appear only in a tool's own documentation
are recorded at all. Where such a row ever exists, its extraction is
`transcribed` like any other, and this record settles nothing beyond that.

It does not decide the locator syntax, the identifier syntax, or the field names,
all of which belong with the source register and the schema.
