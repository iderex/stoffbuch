# Quantities, units and the condition vocabulary

Every number in the register is a quantity with a unit, measured under
conditions. This record decides where quantities are declared, how a unit is
written, whether the register stores what a publication printed or a normalised
form, and what a condition block may contain.

## The quantity register

Quantities live in `register/quantity/`, one file per quantity, in the same
tree as the rows rather than in code. A quantity is data because the set of
quantities will grow for as long as this project exists, and a set that grows in
code makes every addition a change to a program.

A quantity declares:

- an identifier, which is what a row names and which never changes
- its dimension, written as an expression over the base dimensions
- its accepted units, a closed set
- its canonical unit, which is the one a caller receives unless they ask for
  another
- the parts of a material identity it is sensitive to, which is what makes two
  rows comparable for this quantity and is a property of the physics rather than
  of the material
- the conditions it requires, and the conditions it accepts but does not require
- its spectral variable, where it has one, and which spectral variables may be
  used to express it

A quantity that is anisotropic declares orientation among the identity parts it
is sensitive to, so the register cannot hold an anisotropic value with no
direction on it and still look complete.

Adding a quantity is adding a file. Nothing in a program changes, and the gate
refuses the file if its accepted units do not all parse, if any of them has a
dimension other than the one declared, or if its canonical unit is not in its own
accepted set. Removing a quantity that any row names is refused.

## The unit expression

The register uses UCUM, the Unified Code for Units of Measure, rather than a
vocabulary of its own.

The reason is that this project has no business maintaining a unit grammar. UCUM
is published, has a formal grammar rather than a table of strings, is case
sensitive so a millisievert cannot be read as a megasievert, expresses compound
units without ambiguity, and is already what a great deal of measurement
software reads. Writing our own would produce a fourth dialect of the same
thing, and the cost of that dialect would land on every consumer.

The costs of taking it are real and are paid deliberately. UCUM codes are not
what a paper prints, so a reader of a raw file sees `cm2/(V.s)` where the paper
printed square centimetres per volt second, and the documentation has to say so.
UCUM is an external specification, so the register pins the version it was
written against and a change of version is a change to the tree rather than
something that happens underneath it. And UCUM covers more than this project
needs, so the accepted set per quantity does the narrowing that the grammar
alone does not.

Which UCUM code stands for each accepted unit is fixed in the quantity file, not
in prose here, and the gate parses every one of them against the pinned grammar.
That is deliberate: a list of codes written into a document drifts against the
grammar that decides them, and no reader can tell when it has.

## Stored as printed, converted on the way out

A row stores the value with the unit the publication printed. The library
converts when a caller asks for another unit.

The alternative, normalising on the way in, puts an arithmetic step between the
publication and the file, and that step is performed once by a person and is
then invisible. A transposed exponent there is indistinguishable from a
correctly transcribed number, and it survives every later review because the
review compares the file against the file.

Storing as printed costs the reader who wants to answer a question by reading
the file alone. Two rows of the same quantity may be in different units and
cannot be compared by eye. That cost is paid because the alternative hides the
error where nothing can find it, and because the conversion, once it is code, is
testable.

The check that catches a conversion error is a test per ordered pair of accepted
units per quantity, against values computed from the defining relation rather
than from this project's own implementation. A round trip is not that test and
must not be mistaken for it: a factor that is wrong by a decade round trips
perfectly, and the near miss a hurried transcription actually produces is
exactly a factor of ten. The round trip is a second and weaker leg, kept because
it catches an asymmetric implementation cheaply.

Two conversions are named cases rather than factors. Temperature is an offset
scale, so a conversion that treats degrees Celsius as a multiple of kelvin gives
a plausible wrong answer over the range this register lives in, and the test for
it uses a temperature where the wrong answer is plausible rather than one where
it is obviously absurd. Wavelength against photon energy against wavenumber is a
reciprocal relation, not a scale at all, so it carries the constant it uses and
a statement of where that constant comes from.

## The condition block

A row carries a condition block. The vocabulary is closed and covers at least:

- `temperature`, the sample temperature, which is not the same as an ambient
  temperature and not the same as a carrier temperature
- `pressure`
- `doping`, which is a species, a concentration and a statement of whether the
  concentration is nominal or measured, because a nominal doping is what the
  process asked for and a measured one is what the sample has
- `illumination`, which covers the dark case explicitly rather than by silence
- the spectral variable, where the quantity has one, named as wavelength,
  photon energy, wavenumber or frequency and stored as printed like any other
  value
- `orientation`, a crystallographic direction, required by every quantity that
  declares itself anisotropic

Adding a term to this vocabulary is a change to the tree and a decision, not
something a row does by writing an unfamiliar key. A key outside the vocabulary
is refused.

## Three states, and only two of them are a row's to write

A condition slot is in one of three states, and the difference between them is
the part existing collections lose.

It is stated when the row carries a value and a unit.

It is `not-stated` when the publication did not say. This is a value a curator
writes, it is not an empty field, and it is not a defect. A great many of the
field's most used tables carry no temperature at all, and a register that could
not record that fact would either exclude those values or invent one.

It is not applicable when the quantity does not depend on it, and a row never
writes this. It follows from the quantity declaration, which lists what it
requires and what it accepts, and anything the quantity does not name is not
applicable to it. The reason a row may not write it is that a curator who could
declare a required condition inapplicable could silence the check that asks for
it, and that is the one move that would make the whole condition block
decorative. A row writing `not-applicable` in any slot is refused.

Nothing is ever assumed. There is no default temperature, no assumed room
temperature, and no convenience helper that supplies one. Room temperature is
itself a range that different publications mean differently, and filling it in
is precisely how the untraceable numbers this project is a reaction to were
made.

What a report prints for each is fixed, so the two cases never read alike:

- `not-stated` prints as `the publication does not state a temperature`
- not applicable prints as `temperature does not apply to this quantity`

## What a check can refuse

- a unit expression that does not parse under the pinned grammar, naming the
  part that could not be parsed
- a unit outside the quantity's accepted set
- a unit whose dimension is not the dimension the quantity declares, which
  catches an accepted set that is itself wrong as well as a row that is
- a condition key outside the vocabulary
- a condition value whose unit has the wrong dimension for that condition
- a condition the quantity requires that is neither stated nor marked
  `not-stated`
- `not-applicable` written in a row
- a quantity file whose canonical unit is not in its own accepted set

## What this record does not decide

It does not decide the field names or the file format, which belong with the
schema. It does not decide what makes two material identities equal, only that a
quantity is where the relevant identity parts are named. It does not decide the
uncertainty attached to a converted value, which belongs with the uncertainty
record.
