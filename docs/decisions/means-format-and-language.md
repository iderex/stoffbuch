# The means: the stored format first, then the language and the toolchain

The repository holds a readme, a notice, five workflow guards and the decision
records beside this one. It holds no code and no data, and every other issue in
the plan assumes a file format, a language and a place for things to live. This
record writes the assumption down so it can be argued with on the merits.

The means question splits in two, and the split is the part specific to this
project. The product is the register. The code is what guards the register and
what reads it, and both of those can be replaced without the register changing.
So the stored format is decided first and separately, because it outlives every
program written against it, and because a consumer in another language has to be
able to read a row without running anything from here.

## The stored format

Four properties decide it. A row has to be reviewable as a diff, because
curation is the work and the review of the diff is where a transposed digit gets
caught. A row nests: an uncertainty statement has components, a provenance chain
is a list of links, and a condition block is a block rather than a column. A row
has to parse the same way in every language, because a register whose meaning
depends on which parser read it has given up the property it exists for. And a
row has to be writable by a person reading a paper, not only by a program.

The stored format is JSON, UTF-8, one row per file, in a canonical form the gate
enforces, with a schema that is a file rather than an argument in a document.
Large tabulated blocks are held beside the row in a columnar text file that the
row points at.

The reasoning. JSON has a small grammar and no type guessing, so a bare token is
never resolved into a type the writer did not intend. A value register cannot
afford a parser that decides for itself what something meant: the nearest
neighbouring hazard is a version string or an alloy fraction silently becoming a
number, and the field this project reads from is full of tokens that look like
numbers and are not. A canonical form costs one formatter and buys a diff that
means something, so a review sees the field that changed rather than a reflow.
The schema is a file, so the check that refuses a bad row and the specification a
contributor reads are one artefact rather than two that drift.

The costs are real and naming them is part of the check. JSON has no comments,
so anything a curator wants to say has to become a field with a name. That is
more work at the schema and better for the reader, because a comment is
invisible to every consumer and a field is not. Hand writing JSON is unpleasant,
so the tooling owes a curator a way to produce a row without counting braces,
and that is separate work rather than a promise made here.

### The schema language

JSON Schema, draft 2020-12, one schema per row kind, held in the tree and
versioned with it.

It is chosen because it is the schema language for this format that has more
than one independent implementation, so a contributor can validate a row with
something other than this project's binary. That is the same property the format
itself is chosen for and it would be odd to give it up one level up.

Its limits are named rather than discovered later. JSON Schema can refuse a
shape, a type, a pattern and a required field. It cannot refuse a dimension
mismatch, a unit outside a quantity's accepted set, a chain with two read marks,
or a cycle. Everything in that second class is a check in the gate, and the
schema is not the gate. A reader who takes a schema pass as a valid row has
misread what the schema claims, and the documentation says so where the schema
is introduced.

### The canonical form

The rule, in full, because a canonical form stated loosely is not one.

- UTF-8 with no byte order mark.
- Line feed line endings, one trailing line feed, no trailing whitespace on any
  line.
- One member or element per line, indented by two spaces per level of nesting.
- Object keys sorted ascending by the byte sequence of their UTF-8 encoding.
- A duplicate key is refused rather than resolved. Two members with one name is
  the case where two parsers legitimately disagree, and no ordering rule saves
  it.
- Strings carry only the escapes the grammar requires, with lower case hex in
  the ones that need `\u`.
- The formatter never reparses a number. A number is written back exactly as its
  token stood in the file, so a rewrite cannot change what a number says.

That last rule needs its reason spelled out, because it is the one a formatter
written from habit gets wrong. A number printed to four significant figures is a
claim about the fourth figure, and `1.1200` and `1.12` are different claims. A
formatter that parses a number into a binary float and prints it back destroys
the trailing zeros, and it does so silently and uniformly across the whole
register.

Preserving the token in the file is necessary and not sufficient, because a
consumer reads the file with an ordinary JSON parser, and an ordinary JSON parser
turns `1.1200` into a double before the consumer sees it. So the format carries a
second rule: any digits whose spelling is part of the claim are held as a JSON
string, constrained by the schema to a decimal pattern, and parsed by the library
into a decimal rather than a float. The schema is where the list of such fields
is written, because that list is a property of the row kinds. What is decided
here is that the list is not empty and that the value, its uncertainty and a
model coefficient are on it.

The cost is that a raw file shows a quoted value where a reader expected a bare
one, and that a consumer who ignores the library and reads the string as a float
gets a float. That cost is paid because the alternative loses a claim the
register exists to carry, in a way no diff shows.

### The tabulated block

A four thousand line array inside a row is not reviewable, and those blocks are
the one part of the register a spreadsheet will legitimately be pointed at. They
are held beside the row in a plain text columnar file: UTF-8, line feed endings,
one header line naming the columns, one record per line, fields separated by a
single tab.

There are no quoting rules and none are wanted. A tab or a line feed inside a
field is refused rather than escaped, because every quoting dialect in this
family of formats is where two readers stop agreeing, and the fields here are
numbers and short identifiers that never need one.

The row points at the block by relative path and by a digest of its bytes. The
path alone is a pointer that cannot be checked; the digest makes the pairing
something the gate reads. It costs a rewrite of the pointer whenever the block
changes, which is not a burden, because a released row is immutable and a
changed block is a new version of the row either way.

### Rejected, with what each costs

YAML is what the nearest existing collection of this kind uses, it is pleasant
to hand write and it carries comments. It costs a large grammar whose resolution
rules differ between implementations, which is the wrong trade for a file whose
whole purpose is that two readers agree. This is the strongest of the
rejections, and the condition that would reverse it is a single normative parser
behaviour the register could test against: if the field's readers converge on
YAML 1.2 with the core schema, and a conformance suite exists that this project
can run its own reader against, the interoperability objection is answered and
the comment support becomes a real gain.

TOML has an unambiguous grammar and comments, and is the pleasantest of the
three to hand write. It costs deep nesting, which this row has, and arrays of
tables get noisy exactly where the provenance chain lives.

A database file as the stored form gives queries for free and costs the diff,
which is the property the review depends on. It is the right shape for a built
artefact and the wrong one for the source, and nothing here forbids shipping one
as an export.

A flat delimited file for everything costs the nesting outright. It is kept for
the tabulated block, where there is no nesting, and rejected for the row.

## The language

Rust, in one workspace holding the schema check, the gate, the evaluator, the
library and the command line.

The reasoning. The gate needs a lock file, one test command and a linter that
can be made to fail the build, and it needs them without this project owning a
build system. The result has to run for somebody who has not installed an
environment first, which for a register people will consult from a script means
a single binary that behaves the same on Windows as elsewhere. The type system
lets a good part of the row's invariants be unrepresentable rather than checked,
which reduces the number of checks that have to exist and be proven.

The costs. The scientific audience for this project reads and writes Python, and
choosing something else raises the cost of an outside contribution from exactly
the people who have the numbers. The statistical work behind the spread between
publications has better ground elsewhere. Compile times are a daily tax on
whoever is building it.

### Version, build tool and lock file

The toolchain is pinned in the tree, at the version this record was written
against:

    rustc --version
    rustc 1.97.0 (2d8144b78 2026-07-07)
    cargo --version
    cargo 1.97.0 (c980f4866 2026-06-30)

The pin is a `rust-toolchain.toml` at the workspace root naming that version and
the components the gate runs, so a fresh clone and the workflow resolve to the
same compiler rather than to whatever is newest. The workspace declares the same
version as its minimum, so a crate that raises the floor fails at the manifest
rather than at a confusing compile error. The build tool is cargo, and
`Cargo.lock` is committed, because this workspace produces a binary an operator
runs rather than a library others depend on, and a floating dependency graph
makes a build unreproducible in exactly the way this project complains about
elsewhere.

Raising the pin is a change to the tree with its own reason, never something
that happens underneath a run.

### Rejected, with what each costs

Python is the strongest rejection. The curation tooling, the digitising work and
the statistics all have better ground there, and the audience can read it. It
costs a runtime the operator installs first and a reproducibility story that
rests on an environment rather than a lock file. The condition that would
reverse it is the statistics: if the work behind the spread between publications
needs machinery that would have to be written from scratch here, the honest move
is a two language split, with the analysis in Python and the format as the
boundary between the two rather than a shared library. That is why the format is
decided first and why it may not leak the language.

Go costs less than Rust in build time and in the difficulty of an outside
contribution, and gives the same single binary. It is rejected because the
invariants this register carries are the kind a type system can hold, and giving
that up buys convenience the project does not need. Nothing outside forces it,
and it would be a reasonable second choice rather than a wrong one.

## The format is specified independently of the language

The format is specified in a document that names no language, and the test that
holds it is a reader written from that specification alone, in another language,
producing the same values from the same file.

That test is a conformance suite in the tree: input files paired with the values
a correct reader produces from them, including the cases where a wrong reader
differs quietly. Trailing zeros in a value. A duplicate key. A key ordering that
is not the canonical one. A tabulated block whose digest does not match. A
number written in exponent form. The suite is data, so a reader in any language
can be run against it, and the obligation this record creates is that at least
one reader not written in Rust is run against the suite before the format is
called stable. Where that suite sits in the tree follows the tree layout
decision and is not fixed here.

Until such a reader has been run, the independence is a design intention rather
than a measured property, and no document in this project says otherwise.

## Answering the means check, in order

### Can the means carry a property a machine can refuse, a proof that runs, and a claim that cites the command behind it

Yes for both halves. The format is
refusable by construction: a canonical form, a schema and a digest are each
something a program either matches or does not. The language gives one test
command over the workspace and a linter that can be made to fail the build, so a
proof runs where the code is rather than in a parallel apparatus. A claim citing
its command is a property of how this project writes, not of the means, and
neither half obstructs it.

### Is anything outside this repository forcing a different means

For the language, no. Nothing outside forces Rust and nothing outside forbids it, and
this record should not pretend the choice was made under compulsion. For the
format, one thing is forced and its surface is small: a consumer in another
language must read a row without running anything from here, which rules out any
bespoke format and any format without an off-the-shelf parser in the languages
this field uses. JSON satisfies that; so would YAML and TOML, which is why the
rejection above is argued on the other properties rather than on this one. The
unit grammar is a second external force, already taken and already pinned, in
`quantities-units-and-conditions.md`.

### Does the means add a language, a runtime or a dependency the tree does not already carry

Yes, and it is the whole cost of this
record. The tree today carries no language at all: five workflow guards written
as shell inside YAML, and markdown. Rust is a new language, a new toolchain to
pin, a lock file to maintain and a compile step in every route. JSON Schema is a
second specification to pin beside the unit grammar. Both are paid knowingly.
Nothing here adds a service, a database or a network dependency, and the absence
of those is deliberate.

### Would the artefact be testable by the suites that already exist

No suite exists yet, so the
question is whether this means produces one suite or several. It produces one:
the schema check, the evaluator, the library and the command line are crates in
one workspace under one test command, and the conformance suite above is data
those tests read rather than a second harness. The shell inside the workflow
guards is the one thing that stays outside it, and it stays because a workflow
step is a forced means held to its smallest surface.

## Naming and numbering of decision records

This is fixed here so that every record produced by the plan matches, and it is
derived from what has already landed rather than invented:

    git ls-files docs/decisions/
    git grep -c '^## What this record does not decide' -- docs/decisions/

Records live in `docs/decisions/`, one file per decision, named in lower case
with words separated by hyphens after the subject of the decision. A record
carries no number and no date in its name. A number implies an order that is not
the order records are read in, and inserting one later either renumbers the rest
or leaves a gap that reads as a missing record; a date answers a question the
version control history already answers better.

A record opens with a single level one heading naming the subject, and its body
is level two sections. It closes with a section headed `What this record does
not decide`, which is what keeps the boundary between records readable and what
stops a reader assuming an answer that was never given.

A record refers to another by its bare file name in backticks, which is what the
landed records do and which survives the file being read outside a web view.

A record is never renamed once merged, because a rename breaks every reference
to it. A record is superseded rather than edited away: the superseding record
names the one it replaces, and the replaced one gains a line under its heading
pointing forward. Ordinary corrections to a record are ordinary changes, since
these are documents in the present tense rather than a register of what was once
true.

## What this record does not decide

It does not decide the field names, the row kinds or their schemas, which are the
schema work. It does not decide where in the tree the register, the conformance
suite or the built artefacts sit, nor how a register file is named, which is the
tree layout decision. It does not decide the crate boundaries inside the
workspace beyond that there is one workspace, nor which linter warnings are
errors, both of which belong with the gate. It does not decide the export
formats, which may be anything a consumer reads, including the database file
rejected above as a stored form.
