# Versioning, immutability and how a row is cited

A paper will cite a value from this register. Years later somebody will follow
that citation, and either it resolves to the same number or the project has
failed at the thing it was built for.

That pulls against the other thing this register is: incremental, built up
slowly, and wrong in places nobody has found yet. Rows will be corrected.
Transcription errors will be found. A source will turn out to have been misread.
A better reading of an ambiguous equation will change every coefficient set
fitted against it.

## The row identifier

A row identifier is opaque. It is the string `sb-` followed by ten characters
from Crockford's base32 alphabet, which excludes the letters that are confused
with digits when read aloud or copied by hand:

    sb-4KQ7N2XJ0P

It is minted once, when the row is first written, and it never changes. It is
never reused, never reassigned, and never derived from anything about the row.

Opaque rather than readable is the awkward half of this decision, so the reason
is set out rather than asserted. An identifier that reads well is one built out
of the quantity, the material and the source, and every one of those is a thing
that can turn out to be wrong. The case the plan named is a row filed under the
wrong polytype, and it is the ordinary case rather than an edge: a curator reads
a paper that says silicon carbide and does not say which one, files it, and a
later reader with the paper in hand establishes the polytype. If the identifier
carried the polytype, the repair would either change the identifier, breaking
every citation of it, or leave the identifier lying, which is worse. An
identifier that means nothing cannot be made wrong by anything the register
learns.

What it costs is real. A diff shows a string nobody can read. Two identifiers
differ in a way the eye does not catch. Nothing about a row can be guessed from
its identifier. Against that: every tool that prints an identifier prints a
readable label beside it, formed from the quantity, the material and the source
year, and that label is a display and is never used to look anything up. A label
in a citation is a courtesy to the reader and carries no guarantee.

The characters are random rather than sequential, and that is a decision rather
than a shortcut. A sequence needs an allocator, and an allocator is a thing two
people working at once both ask, or a counter in the tree that every branch
increments to the same number. Ten Crockford characters are fifty bits, which is
enough that a collision is not the failure mode worth designing against, and the
gate refuses a duplicate identifier anyway, so the property is checked rather
than trusted.

## Versions

A version is an integer, starting at one, incremented by one for each released
correction to the row.

Not a date, because two corrections in one day need an order. Not a digest,
because a reader has to be able to see that version three supersedes version two
without resolving either. The whole guarantee a version carries is that a higher
number is later and supersedes what is below it, and an integer is the only form
that shows that on its face.

A row at a version is written with an at sign between the two:

    sb-4KQ7N2XJ0P@3

## Immutability

A row is immutable once released. A correction produces a new version of the row
and never an edit to an existing one.

Released is the important word. Before a row has appeared in a release it is an
ordinary change in an ordinary branch and it is edited like anything else. From
the release onward, the bytes of that version of that row do not change again,
because somebody may hold them.

Nothing is deleted. A row that should never have existed is not removed; it gains
a final version marked withdrawn, carrying the reason and, where it duplicates
another row, the identifier of the survivor. A withdrawn version still resolves,
so an old citation still finds something, and the something it finds says what
happened. The evaluator and the library refuse to compute over a withdrawn
version, which is the difference between a citation resolving and a number being
usable.

The new version carries a correction block: a cause from a closed vocabulary, and
a sentence a person wrote. The vocabulary covers at least a transcription error,
a misread source, a better reading of an equation, a source retracted or
corrected by its publisher, and an identity correction. The sentence is required
even where the cause seems to say everything, because a cause is a category and
the reader wants to know which digit moved.

The polytype case above lands here as an identity correction. It is a new
version of the same row, not a new row, because the row is still the same claim
about the same publication at the same locator; what changed is what this
register believed about the sample. Making it a new row would break the citation
and would leave two rows in the register that a comparison could count as two
determinations of the same thing.

## Supersession is written into the record as well as into version control

Both, and the duplication is only apparent.

Version control already holds the history and is already auditable, so writing
it twice would be waste if the two held the same thing. They do not. A consumer
that received a file has no version control, and a row that cannot say what it
supersedes is not self describing. Everything about who made the change and when
stays in version control and is never copied into the row; everything about
which claim replaces which, and why, is in the row and is what a consumer reads.

The pointer runs forward only. The new version carries the identifier and version
of what it supersedes. The old version carries nothing, because writing a
backward pointer into it would be an edit to a released row, which the rule above
forbids. The backward direction is computed: the gate builds the index across the
release and knows, for every version, what supersedes it.

That is a consequence worth stating plainly rather than discovering later. There
is no field in a released row that says it has been superseded. A consumer
holding one file alone cannot know, and a consumer holding the release can.

## Citation

A citation names three things: the row, the version, and the release.

    stoffbuch, row sb-4KQ7N2XJ0P@3, release 1.4.0

The release identifier is what makes a citation resolvable years later, because
the release is the artefact that was published and archived, and the row and
version are how to find one thing inside it. The form of a release identifier is
fixed with the release route rather than here, and `1.4.0` above stands for
whatever that form turns out to be. What this record requires of it is that it
exists, that it is ordered, and that a released register can be obtained by
naming one.

A citation that gives a row and a release but no version resolves to whatever
version of that row the named release contained. That is well defined, so it is
allowed, and it is what a paper citing a whole release will naturally write.

A citation that gives a row and a version but no release resolves to the row, and
is weaker: it says which claim is meant but not which published artefact contains
it, so a reader has to search releases to find one. It is allowed and the
documentation says what it does not carry.

A citation that names only a row names nothing stable, and the documentation says
so in those words rather than treating it as a shorthand.

A worked citation of a single row, of the shape a paper could print:

    The silicon band gap value used here is stoffbuch row sb-4KQ7N2XJ0P@3,
    release 1.4.0, at https://github.com/iderex/stoffbuch

Whether a release also carries a persistent identifier from an archive, which is
what a reviewer in this field will ask for, is not decided in this project yet.
The citation form above is written so that adding one adds a field rather than
changing the form.

## What a consumer sees when its pinned version has been superseded

It sees the version it pinned, and it is told.

The library resolves the pinned version and returns it. It never silently returns
the newer one, because a consumer that pinned a version did so to get a stable
number, and quietly changing it is the failure this whole record is against. It
never silently returns the old one either, because a consumer that pinned a
version two years ago and has not looked since is exactly the reader who needs to
know a correction exists.

So the result carries a warning naming the superseding version and the correction
cause, and the run exits successfully with the value it was asked for, which is
what `error-and-failure-policy.md` sets out for a usable result with a weaker
claim under it. A caller who would rather stop than carry a superseded number
into a figure asks for warnings to be fatal, and gets a refusal instead.

This is something the library does. It is not a paragraph in the documentation
asking the reader to remember to check, because a reader who has to remember will
not, and the moment they need reminding is a run they are not watching.

A pinned version that the release being read does not contain is a refusal rather
than a warning. There is nothing to return, and returning a neighbouring version
would be inventing an answer.

## What a check can refuse

- an identifier that does not match the form, including one carrying a character
  outside the alphabet
- two rows in the tree with the same identifier
- two records with the same identifier and the same version
- a version that is not a positive integer, or a set of versions for one row with
  a gap in it
- a version of one that carries a supersession pointer, and a version above one
  that does not
- a supersession naming a row that is not in the register, or a version of that
  row that does not exist
- a supersession pointing at anything other than the version immediately below
- a new version whose correction block has no cause, a cause outside the
  vocabulary, or no sentence
- a withdrawn version whose cause is duplication and which names no survivor
- a row at a version that a release contained whose bytes differ from what that
  release published, which is the check that gives immutability force and which
  compares the tree against the released artefact rather than against a
  convention

That last one is the one worth building first. Every other refusal above catches
a mistake in a new row, and this one catches the change to an old row that no
review would look for.

## What this record does not decide

It does not decide the field names or where the identifier and version sit in a
record, which belong with the schema. It does not decide the release identifier
form, the release route or the version numbering of the tools, all of which
belong with releases. It does not decide whether a release is deposited with an
archive that mints a persistent identifier. It does not decide what a derived row
does when one of its inputs is superseded, which is settled in `record-kinds.md`
as staleness and is not restated here.
