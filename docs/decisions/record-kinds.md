# What a row is: measured value, fitted model, derived value

This record settles the boundary between the three kinds of thing the register
holds. It does not settle the schema. Whether the three kinds are three files,
three record kinds in one schema, or one kind with a discriminator is decided
where the schema is decided.

## The word "row"

`Row` is the umbrella term. Every record in the register is a row, and every row
is exactly one of the three kinds below. Where a document or an issue in this
project says "row" without qualification it means any of the three, and where it
means one kind it names that kind.

Two narrower words are already in use and keep their meaning. A `coefficient
set` is a fitted-model row, named that way when the coefficients rather than the
row are the subject of the sentence. A `value` is whatever a caller receives
after asking a question, which may come from a measured row directly or from
evaluating a coefficient set at a condition.

## The three kinds

### Measured value

Asserts that a named publication reported this number, for this material, under
these conditions, obtained by this method.

May not assert that the number is correct, that it is the best available, or
that it holds at any condition other than the one recorded. It may not carry
model coefficients, and it may not carry a value this project computed from it,
including a unit conversion.

The one sentence that tells it from the other two is that it is a statement
about what a publication printed, and it does not change when anything else in
the register changes.

A measured value is tied to one source. If the publication is later withdrawn or
shown to be wrong, the row stays and is marked, because the row was never a
claim that the number was right.

### Fitted model

Asserts that a named publication reported these coefficients for this named
model form, valid over this stated range, and that the form is the one this
register identifies under that name.

May not assert that the coefficients are measurements, and may not omit what
they were fitted to. Where the publication does not say what was fitted, the row
says that the publication does not say, which is a statement rather than an
empty field. It may not name a form that does not exist in the form register,
and it may not carry a coefficient outside the form's parameter list.

The one sentence that tells it from the other two is that it produces a number
only after something evaluates it at a condition, and the number it produces
depends on which version of the form is used.

The residual of the fit, where the publication reports one, belongs to the row.
A fit whose residual is unreported is a weaker thing than a fit whose residual is
reported, and the register distinguishes the two rather than treating a missing
residual as a good one.

### Derived value

Asserts that this project computed this number from these inputs, which are rows
in this register named by identifier and version, using this named recipe.

May not exist without its inputs being present in the register. May not be
transcribed from anywhere: a number taken from a publication is a measured value
even when the publication itself computed it, because what this register can
check is its own arithmetic and not somebody else's. May not carry a source
citation of its own; its provenance is its inputs, and their sources are reached
through them.

The one sentence that tells it from the other two is that it is the only kind a
machine can recompute, and the only kind that changes when something else in the
register changes.

## The boundary cases

### A publication reports a measured value and a fit to it in the same table

Two rows, both citing the same source, at different locators. The measurement
gets a measured-value row at the table and row where the number is printed. The
fit gets a fitted-model row at the equation or the caption where the
coefficients are printed. The fitted-model row names the measured-value row as
what it was fitted to, by identifier, which is the ordinary case and the reason
that field exists.

The curator does not merge them, and does not drop the measurement because the
fit reproduces it. The measurement is what the field can re-examine; the fit is
somebody's reading of it.

### A coefficient set with no underlying data cited at all

Still a fitted-model row. The field saying what it was fitted to carries the
explicit statement that the publication gives none, and the row is complete.

This is not a defect to be repaired by guessing. A great deal of the literature
this register is about prints coefficients with no data behind them, and a
register that refused those rows would exclude exactly the numbers whose
untraceability is the problem. What the register owes instead is that the
absence is visible and countable, so a report can say how many coefficient sets
in a set rest on nothing this project can reach.

### A value this project derived that a later publication then cites back

Two rows, and the second one is a measured value. The derived row stays what it
is. When a publication quotes it, that publication becomes a source like any
other, and a curator transcribing from that publication creates a measured-value
row citing it.

The two rows are not merged and neither is removed, because they assert
different things: one says this project computed a number from named inputs, the
other says a publication printed a number. Where the second is recognisably the
first coming back, the measured-value row records that in its extraction, so a
comparison does not read the pair as two independent determinations. The failure
this avoids is a register that inflates the apparent agreement of the field by
counting its own output as evidence.

## Derived values as inputs to derived values

Permitted. A pooled estimate over unit-converted values is a natural and useful
thing, and forbidding it would push the conversion into a curator's arithmetic,
which is the thing the register is built to avoid.

What stops a cycle is a check rather than a convention. The inputs of every
derived row form a directed graph over row identifiers at fixed versions. The
gate builds that graph and refuses it if it is not acyclic, naming the rows on
the cycle. A row naming itself, directly or through any number of steps, is the
same refusal.

## What happens to a derived value when an input changes

An input never changes in place. A correction produces a new version of the
input row, and the derived row goes on naming the version it was computed from.
So a derived row is never silently wrong, and it can be one of two things
instead.

It is **inconsistent** when recomputing its recipe over the versions it names
does not reproduce the number it holds. That is a defect in the tree and the
gate refuses it, naming the row, the recipe and the two numbers. This is the
check that makes the derived kind worth having, and it is the reason a derived
row must name its inputs by identifier and version rather than describe them.

It is **stale** when every version it names is reachable and reproduces its
number, but at least one of those versions is no longer the current version of
that input row. A stale row is a correct statement about the versions it names,
so it is reported rather than refused, and the report names the input that moved
and the version that supersedes it. Retiring the staleness means writing a new
version of the derived row against the new inputs, which leaves the old one
readable for anything that cited it.

Both conditions are decided by reading the tree. Neither rests on anybody
remembering that a recomputation is owed.

## What this record does not decide

It does not decide the identifier form, the version form, or what a citation of
a row looks like. It does not decide the recipe vocabulary for derived values
beyond requiring that a recipe is named and that naming it is enough for a
machine to recompute the row. It does not decide whether the register publishes
a recommended value; if one is ever published it is a row of one of these three
kinds under its own provenance, and this record does not choose which.
