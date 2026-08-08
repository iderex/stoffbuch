# Where the register lives in the tree, and how a file is named

The register is the product. People will link to individual files, scripts will
glob them, and a reorganisation later breaks both, so where a file sits and what
it is called is an interface and not an implementation detail.

Four things have to be settled together, because each one constrains the others.
Whether a record is one file or many records share a file, which decides how a
diff reads and how a merge behaves. How a file name is derived from what the
record is about, and what happens to that name when the record is corrected.
Where sources sit relative to the rows that cite them, since a source is
referenced by many rows and duplicating it is how a citation drifts. And where
the large tabulated blocks go, since those are the files a person will want to
download on their own.

`means-format-and-language.md` fixed the stored format and left this question
open by name. `versioning-and-citation.md` fixed the identifier and the version,
and almost everything below follows from those two rather than being chosen here.

## The tree

    register/rows/
    register/sources/
    register/forms/
    schema/
    conformance/

`register/` holds the product and nothing else. A file under it is either a row,
a source, a form, or a tabulated block owned by one of those. Nothing about this
project's code, its tests or its documentation lives under `register/`, so a
consumer who wants the register and none of the rest can take that one directory
and have a complete thing.

The three registers inside it are separated because they hold different kinds of
artefact with different identifiers and different lifetimes, not because of what
a row is about. All three row kinds from `record-kinds.md` share one tree, and
the reason is the identifier. A row identifier is opaque, so it carries no hint
of which kind the row is, and a reader holding a citation would have to search
three directories to resolve one. One tree makes resolution a path a reader can
construct rather than a search, and it is the same property the identifier was
made opaque for.

`schema/` and `conformance/` sit at the root rather than inside `register/`
because neither is part of the published register. The schema describes the
register and the conformance suite tests readers of it, and a consumer taking
`register/` alone should get data rather than data mixed with its specification.

## One file per row, per version

One record per file. Every version of a row is its own file, and every version
that has ever existed stays in the tree.

The first half is what `means-format-and-language.md` already decided when it
chose one row per file, and it is what makes a diff readable and a merge between
two contributions in the same week a thing that mostly succeeds.

The second half is not a separate choice. It falls out of what
`versioning-and-citation.md` requires a check to be able to refuse. A set of
versions for one row with a gap in it cannot be recognised unless every version
is present to be counted. A withdrawn version still resolves, which means it is
still somewhere a resolver can read. The check that compares a row at a version
against the bytes the release published needs those bytes in the tree to compare.
And two records with the same identifier and the same version is a refusable
thing, which only makes sense in a tree where two records may share an identifier
and differ in version.

The cost is paid at the diff, and it is the cost worth naming because it works
against the argument the format was chosen on. A correction adds a file rather
than changing one, so the review sees a new file in full instead of the one field
that moved, which is exactly the view a transposed digit hides in. What answers
it is a command that diffs two versions of a row against each other, and that is
owed by whatever produces it rather than promised here. Until such a command
exists, a reviewer diffs the two files by hand and the documentation says so
rather than implying the review is easier than it is.

The alternative was one file per row holding its current version, with the
history left to version control. It reads better in a diff and it fails the thing
the register exists for: a consumer who received a file has no version control,
and a release that cannot be checked against its own published bytes has an
immutability rule with nothing behind it.

## How a file is named

A row at a version lives at

    register/rows/sb-4KQ7N2XJ0P/sb-4KQ7N2XJ0P@3.json

A directory per row, named for the identifier. Inside it, one file per version,
named for the identifier and the version in the form
`versioning-and-citation.md` already fixed for a citation, with the format's
extension.

The identifier appears twice and the redundancy is deliberate. The directory is
what puts every version of one row, and every tabulated block those versions own,
in one place a person can look at. The file name is what a file carries with it
when it leaves the tree, and a block or a row downloaded on its own and called
`3.tsv` is a file nobody can place again. It also gives the naming check
something to bite on: a file whose name disagrees with the identifier inside it
is a mistake a check can see, and it cannot see it if the name says nothing.

The at sign is taken from the citation form rather than invented, so a reader
holding `sb-4KQ7N2XJ0P@3` can construct the path by hand without consulting
anything. It is a legal character in a path on every platform this project
supports and needs no quoting in an ordinary shell. Where a tool does treat it
specially the path can be quoted, which is a smaller cost than a second spelling
of a version that a reader has to translate.

Nothing in the tree is sharded into subdirectories by the first characters of an
identifier. A shard is a rule contributors get wrong and tools have to
reimplement, and it buys nothing while a plain listing of `register/rows/` is
still a usable thing. The condition that would introduce one is that listing
becoming unusable in ordinary tools, which is a measurement rather than a guess,
and the record that introduces a shard states the measurement it was made on.

## What happens to a name when an identity is corrected

Nothing.

`versioning-and-citation.md` settles this and this record only spells out what it
means for a file. A row filed under the wrong polytype is corrected by a new
version of the same row, because the row is still the same claim about the same
publication at the same locator and what changed is what this register believed
about the sample. The identifier does not change, so the directory does not
change, and the correction adds `sb-4KQ7N2XJ0P@4.json` beside what is already
there.

That is the whole reason the name is derived from an opaque identifier rather
than from the material. A name built from the quantity, the material and the
year would have to be either changed or left lying the first time an identity
was corrected, and both of those break something a citation rests on.

## Sources and forms

A source lives at `register/sources/<source-identifier>/`, and a form at
`register/forms/<form-identifier>/`, under the same shape: a directory per thing,
one file per version inside it, named for the identifier and the version.

Sources sit in their own tree and rows point at them by identifier. A source is
cited by many rows, and a citation copied into every row that uses it is a
citation that drifts the first time one copy is corrected. That is the failure
this project is about, so it is not a thing to be careful about; it is a thing
the layout makes impossible.

Forms are versioned already, because a coefficient set names a form version and
an evaluator is required to refuse a form version it does not implement. Whether
a source carries a version at all, and what a source identifier looks like,
belongs with the source register rather than here. What this record fixes is that
whatever that identifier turns out to be, it is opaque and stable for the same
reason a row identifier is, and it names a directory rather than a file.

## Tabulated blocks

A tabulated block lives beside the version of the row that owns it, in that row's
directory:

    register/rows/sb-4KQ7N2XJ0P/sb-4KQ7N2XJ0P@3.n-and-k.tsv

The identifier and version come first so the file says which row version it
belongs to when it is read on its own, and the part between that and the
extension names the block, so a row version that owns more than one block has no
collision and a reader can tell them apart.

A block belongs to a row version and not to a row. `means-format-and-language.md`
has the row point at its block by relative path and by a digest of the bytes, and
a released row is immutable, so a changed block is a new version of the row
either way. Naming the block for the version it belongs to makes that visible in
a listing rather than only inside the file.

The block format is already a plain columnar text file with one header line and
tab separators, which is what makes it readable with ordinary tools and with a
spreadsheet, and nothing here narrows that.

## The conformance suite

`conformance/` at the root, as data rather than as a test in any language.

`means-format-and-language.md` created this obligation and left the place to this
record. The suite is input files paired with the values a correct reader produces
from them, and its whole point is that a reader written in another language can
be run against it. Putting it inside the workspace would make it look like a
fixture directory belonging to one language's tests, and somebody would
eventually move a Rust helper into it. At the root it is what it claims to be,
and a reader in any language reaches it by a path that names nothing about how
this project is built.

It is not under `register/` because it is not part of the published register, and
a conformance input is a file deliberately shaped to break a reader, which is the
last thing that should sit among rows a consumer takes at face value.

## What is not in the tree

Nothing this project builds is tracked. The build directory sits at the root and
is ignored, and its entry belongs with the workspace rather than with this
record. Release artefacts, including the packaged snapshot of the register, are
produced by the release route and are not files in the tree.

The generated parts of the documentation are the case worth stating rather than
leaving to be discovered. A coverage report or a worked example that is generated
from the register is written into the documentation by the route that generates
it, so it is tracked, and it is tracked because a published number that nobody
can see change is a published number that drifts. What makes that safe is that
the route regenerates it and the gate notices a difference, not that a person
remembers.

## Finding every row about one material

A person has to be able to find every row about one material with a single
command, and with opaque identifiers that command is a search of the row contents
rather than of the names. That is the cost of opaque identifiers and it was
accepted when they were chosen.

The command is a plain search over `register/rows/` for the material identity
field, and it needs no code from this project. What it cannot be written down as
yet is the literal, because the field name is part of the schema and the schema
is not written. The command goes into the documentation with the schema that
gives it its field name, and this record does not invent one to have something to
print.

## What a check can refuse

- a file under `register/` whose path does not match one of the shapes above
- a file whose name carries an identifier that is not the one inside the file
- a file whose name carries a version that is not the one inside the file
- a row directory whose name is not the identifier every file inside it carries
- a row file outside a directory, or a directory under `register/rows/` holding
  no row file
- a tabulated block naming a row version that does not exist
- a tabulated block in a directory other than the one belonging to the row that
  points at it
- a file under `register/` with an extension the format decision does not name

## What this record does not decide

It does not decide any field name, which belongs with the schema, and that
includes the field the material search reads. It does not decide the source
identifier form or whether a source is versioned, which belong with the source
register. It does not decide what a form identifier looks like, which belongs
with the form register. It does not decide the release identifier form or where
a released artefact is published, which belong with releases. It does not decide
the crate boundaries in the workspace or where test fixtures sit, neither of
which is under `register/`. It does not decide the digest algorithm a row uses to
point at its block, which is part of the format.
