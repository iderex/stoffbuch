# Material identity, and what makes two rows comparable

Two rows can be compared when they are about the same material. Deciding when
that is true is the join key of the whole register, and getting it wrong makes
rows unmergeable in a way no later work repairs.

The problem is not chemistry. It is that the field's own naming is ambiguous in
ways that change the number. Silicon carbide is several materials with different
band gaps and the polytype is what separates them. Silicon dioxide grown in a
furnace and silicon dioxide deposited from a gas behave as different materials
for an optical constant and as the same material for a density, and the paper
rarely says which question its reader is asking. An alloy carries a composition
that is itself a measured quantity with its own uncertainty. Amorphous material
has no structure to name and its properties depend on how it was made, which is
a paragraph rather than a field.

Two things pull against each other. An identity fine enough to be honest makes
the register sparse, because every row becomes unique and nothing is ever
comparable. An identity coarse enough to be useful throws away the distinction
that explains why two publications disagree, which is the thing the register
exists to show.

## The parts of an identity

An identity has three parts and no others.

The composition. What the material is made of, as elements and their
proportions, including the alloy fraction where there is one. This is what makes
silicon not germanium and it is the part nothing else can substitute for.

The phase. The structural arrangement the field names: a polytype where the
material has them, a crystal structure where it does not, or amorphous where
there is no long range order to name. This is what separates the several
materials the field calls silicon carbide, and it is the part most often left
implicit in a paper.

The crystalline state. Single crystal, polycrystalline or amorphous. This is not
the same question as the phase and collapsing the two loses the case that
matters most in practice, which is a polycrystalline film of a material whose
phase is perfectly well named.

Everything else about the sample is the preparation description, which is
covered below and is never part of the key.

## What is excluded, and why

Doping is a condition. It varies continuously, it changes some quantities and
not others, and a row states it anyway. Making it part of identity would mean
that a mobility measured at one concentration and a mobility measured at another
are about different materials, which would make the doping dependence of
mobility uncomputable from this register.

Temperature, pressure, strain and illumination are conditions for the same
reason. They vary continuously and the register would hold one row per value of
them.

Orientation is a condition, and this needs saying plainly because
`quantities-units-and-conditions.md` places it in the condition vocabulary and
also calls it an identity part in one sentence. It is a condition, it lives in
the condition block, and it has one home. The reason is that orientation is a
property of the direction a measurement was taken in rather than of the material
in the sample holder, and a register in which a wafer becomes three materials
depending on which way the light went through it has lost the join it needs. The
sentence in that record is read as the comparability rule set out below, under
which a quantity that declares itself anisotropic requires orientation to match
before two rows may be compared. That rule gives it everything the sentence
wanted and puts the value in one place.

The preparation description is excluded from equality and is the important
exclusion. It carries what the publication said about how the sample was made:
the growth or deposition method, the temperatures, the precursors, the anneal,
the substrate, whatever was printed. It is free text, it is searchable and
quotable, and no equality rule reads it.

It is excluded because it cannot be normalised. Two papers describing the same
deposition write it differently, at different length, with different things left
out, and no vocabulary this project could write would make those two strings
equal. An identity that included it would make every film unique, every set of
comparable rows a set of one, and the disagreement report empty. That is the
sparse failure named at the top, arriving through the one field most likely to
cause it.

What the exclusion costs is stated rather than argued away. Two rows the field
would call different materials can have one identity here, and the register
answers with a spread rather than with a distinction. That answer is honest and
it is weaker than the distinction would be.

## Comparability is computed, not stored

No row carries a list of the rows it may be compared with, and no set of
comparable rows is stored.

Two rows are comparable for a given quantity when their identities match on the
parts that quantity declares relevant, and their conditions match on the
conditions that quantity requires. Both lists live in the quantity file, in
`register/quantity/`, which is where `quantities-units-and-conditions.md` puts
them. That places the decision on the quantity, where the physics is, rather
than on the material, where it is not.

The consequence is that comparability is a property of a question rather than of
a pair of rows. Two rows on the same silicon sample are comparable for a density
and not comparable for a mobility, because the mobility quantity declares doping
required and the density quantity does not. Storing the answer would mean
storing it per quantity per pair, and it would go stale the moment a quantity
file changed.

Storing it is rejected for a second reason. A stored comparability set is a
derived thing, and this register already has a kind for derived things with a
staleness rule attached, in `record-kinds.md`. Anything that genuinely needs to
be cited as a fixed set can be written as a derived row and inherits that
machinery, so nothing is lost by keeping the ordinary case computed.

## Composition, and the one continuous number inside identity

The elements present and the sublattice each sits on are matched exactly. A
material containing aluminium and one containing indium are never the same
identity, however close the fractions.

The alloy fraction is the one continuous number inside an identity, and exact
equality on it would be useless: no two publications grow a sample to the same
fraction to the last digit, and a rule demanding they do makes every alloy row
unique. So the fraction is matched under a tolerance the quantity declares,
exactly as a continuous condition is. A row admitted into a comparable set under
that tolerance rather than at the fraction asked for is a warning rather than a
refusal, in the class `error-and-failure-policy.md` places that case in.

A fraction is written as a mole fraction on a named sublattice, a decimal
between zero and one, held as a string of digits under the rule in
`means-format-and-language.md`, so a fraction printed as `0.300` keeps the claim
its trailing zeros make. The composition is written with each constituent and
its fraction as fields rather than as a formula string, because a formula string
is a small language this project would then own.

Each fraction carries a basis saying whether the number is nominal, meaning what
the growth was asked for, or measured, meaning what somebody determined on the
sample. The basis is not part of equality: a nominal `0.30` and a measured
`0.30` are the same identity. It is reported, because a comparison resting on
nominal fractions is weaker than one resting on measured ones, and the reader
who is told which is which can act on it.

The uncertainty of a fraction does not belong to the identity. An identity is a
key and a key with an interval on it is not a key. Where the publication gives
an uncertainty on the composition, it is recorded beside the fraction and it is
read by the comparability tolerance rather than by the equality rule, so a
composition known to a tenth and a composition known to a hundredth are not
treated as equally precise statements when a set is built.

Where a publication gives both a nominal and a measured fraction and they
disagree, the row carries both. The fraction is the measured one, because the
sample is what was measured rather than what was asked for, and the nominal one
is recorded in the preparation description where the reader can see the
discrepancy. A disagreement wider than the quantity's tolerance is reported
rather than refused. It is a fact about that sample, and a rule that refused the
row would push a curator to drop one of the two numbers, which is the one
outcome worse than recording the disagreement.

## The four cases

### Silicon carbide, where the polytype is the material

Composition silicon and carbon in equal proportion. Phase `4H`, or `6H`, or
`3C`, named from the field's own polytype notation. Crystalline state single
crystal.

Two rows differing only in the polytype are not the same identity and are never
compared, which is the whole point of the case.

What it loses. A real sample of one polytype contains stacking faults and
inclusions of another, and the fraction of those changes the number. The
register cannot express that in an identity, and it goes in the preparation
description where the publication reported it.

### Silicon dioxide, thermally grown against deposited

Composition silicon and oxygen in the ratio the publication states. Phase
amorphous. Crystalline state amorphous. The furnace oxide and the deposited film
have the same identity, and their preparation descriptions differ completely.

So they are compared, and for a refractive index they will disagree far beyond
their stated uncertainties. That is not a failure of the identity. It is the
disagreement report doing its job with the preparation description visible
beside every row, and a reader who sees the deposited rows sitting apart from
the grown ones has learned the thing the paper's own comment field could not
tell them.

What it loses is the ability to say in the key that these are different
materials for one quantity and the same for another. The register says it as a
spread instead of as a distinction, and that is weaker.

The case where the field has a real name for the difference is not this case.
Crystalline quartz and fused silica differ in phase and in crystalline state,
and the identity separates them already, with no appeal to how either was made.

### An alloy at a stated fraction

Composition aluminium and gallium on the group three sublattice at their
fractions, arsenic on the other. Phase zincblende. Crystalline state single
crystal.

A row at `0.30` and a row at `0.32` are the same identity for a quantity whose
tolerance is wider than `0.02` and different identities for one whose tolerance
is narrower, and a set built under the tolerance says so in a warning.

What it loses. The fraction varies across a real wafer and the publication
reports one number for it. The register holds the number the publication
attributed to the sample and nothing about the gradient, unless the publication
described it, in which case it is preparation.

### The amorphous and poorly specified case

Composition silicon, with hydrogen at the fraction the publication states, or
with no hydrogen fraction at all where it states none. Phase amorphous.
Crystalline state amorphous.

Two hydrogenated amorphous silicon films made in different laboratories have the
same identity here and can have properties that differ by orders of magnitude.
Everything that decides those properties, the deposition temperature, the
dilution, the hydrogen bonding configuration, the defect density, is preparation
and is excluded from equality.

What it loses is most of what a specialist means by the material. The register
does not hide that. A comparable set over an amorphous identity is expected to
show a spread far wider than the stated uncertainties, the report says so in
words, and the preparation description of every row in the set is printed beside
the values rather than summarised away. Whether the register ever publishes a
single value for such a set is not decided here and not decided anywhere yet;
what is decided is that the spread is reported and that no rule in this record
narrows it.

## Worked pairs

These are the fixtures the schema check carries, so the rule is exercised by
data rather than restated in a test name.

Silicon carbide phase `4H`, single crystal, against silicon carbide phase `6H`,
single crystal. Not the same material. The phase differs and the composition
does not, which is the case that would be lost by an identity built on
composition alone.

Silicon, single crystal, boron at ten to the sixteen per cubic centimetre,
against silicon, single crystal, phosphorus at the same concentration. The same
material. Doping is a condition, so the identities are equal. They are still not
comparable for a carrier mobility, because that quantity requires doping to
match and the species differs, and the distinction between an identity that
matches and a comparison that is refused is exactly the distinction this record
is built on.

Silicon dioxide, amorphous, thermally grown at one thousand degrees Celsius in
dry oxygen, against silicon dioxide, amorphous, deposited from silane and
nitrous oxide. The same material. The preparation descriptions differ and no
equality rule reads them.

Aluminium gallium arsenide at aluminium fraction `0.30`, zincblende, single
crystal, against the same at `0.32`. The same material for a quantity declaring
an alloy tolerance of `0.03` or wider, and not the same for one declaring
`0.01`. The pair exists to prove the tolerance is read from the quantity and not
from a constant somewhere in the code.

## What a check can refuse

- an identity missing any of the three parts
- a phase outside the vocabulary the register holds for that composition
- a composition whose fractions on one sublattice do not sum to one
- a fraction outside zero to one, or written as a number rather than as digits
- an alloy fraction on a composition that declares no sublattice for it
- a row whose quantity declares an identity part relevant that the row's
  identity does not carry
- a quantity file naming an identity part that is not one of the three

## What this record does not decide

It does not decide the field names or the file layout of an identity, which
belong with the schema. It does not decide the phase vocabulary itself, which
grows with the materials and is data in the register rather than a list here. It
does not decide the tolerance values, which are per quantity and live in the
quantity files. It does not decide what a comparison computes once a comparable
set exists, which belongs with disagreement. It does not decide whether a value
that appears only in a tool's own documentation is recorded at all, and nothing
above turns on the answer.
