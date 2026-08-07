# What this register does not hold

A register of material parameters has no natural edge. Every neighbouring
quantity is arguably in scope and every neighbouring project overlaps somewhere.
A plan that does not draw the line now draws it later under pressure and badly.

Four boundaries, each with the examples that make it usable by somebody who did
not write it.

## Against the projects this register feeds

A parameter that is a property of a material belongs here. A parameter that is a
property of a calculation belongs to whoever did the calculation.

Inside: the mass density of a material, with its conditions and its source. The
stoichiometry of a compound, including the alloy fraction where there is one.

Outside: a fitted stopping cross section for a projectile in a target, which is a
property of the fit and of the pair rather than of the material, and which
belongs to the effort that fitted it. A range table calibrated so that a
particular process simulator reproduces a particular set of measurements, which
is a property of that simulator.

The density and the composition a stopping fit was made for belong here. The fit
does not, and it cites the density and the composition from here.

## Against structural and computed materials databases

Large collections of computed properties exist, are well maintained, and are not
what this project is for. Duplicating one would be work with no argument behind
it.

Inside: a measured lattice constant, with the temperature it was measured at. A
measured band gap with its method and its sample.

Outside: a bulk modulus produced by a first-principles calculation and never
measured. A predicted property of a compound nobody has made.

A published computed value may appear here, and only under the following, which
is the part of this boundary that had to be decided rather than assumed.

It enters as one row at a time, never as an import of a collection. Its method
field names the calculation, not merely the fact that it was one, because a
number from one method and a number from another are not comparable and a bare
mark saying "computed" hides that. It exists for a reason written into the row,
and the reason has to be that a measured row or a fitted model already in the
register cannot be read without it. It is not a derived value, because this
project cannot recompute somebody else's calculation and the derived kind is
defined by being recomputable. See `record-kinds.md`.

What a report does with it is the other half of the marking. A comparison
separates computed rows from measured ones and reports the two counts, and never
folds them into one spread, because a spread that mixes them is a statement about
neither. The coverage report counts them for the whole register, so the size of
this exception stays visible instead of growing quietly.

## Against the neighbouring registers with their own kind of quantity

Line data for atoms and molecules, measurement histories, uncertainty budgets:
each of those is somebody's whole project, and several of them carry uncertainty
and provenance as seriously as this one does.

This register holds bulk and interface properties of solids.

Inside: the thermal conductivity of a solid. An interface trap density at a named
interface, which is a property of the pair of materials rather than of a device.

Outside: a spectroscopic line list. A table of atomic transition probabilities.

Where a concept is genuinely shared with one of those registers, such as the
shape of an uncertainty statement, the answer is a shared specification rather
than a shared scope. Adopting somebody's uncertainty vocabulary is cheap.
Adopting their subject matter is not.

## Against device and process data

A parameter that only means anything for a particular device geometry or a
particular process recipe is not a material parameter, however often the field
quotes it as one. This is the boundary that will be pushed hardest, because the
numbers people want are frequently exactly these.

Inside: the carrier mobility of a doped bulk material at a stated temperature and
doping. The refractive index of a deposited film, with the preparation described
and the preparation kept out of the identity that decides comparability.

Outside: the threshold voltage of a transistor of a particular geometry. An etch
rate for a particular recipe on a particular tool.

The honest part of this boundary is that a film's properties depend on how it was
made, and that the register admits such rows with the preparation visible rather
than pretending the dependence away. What it will not do is admit a number whose
subject is a device.

## The test for a proposed quantity

Is it a property of the material itself, such that two laboratories given the
same material under the same stated conditions should obtain the same number,
with no device, process recipe or calculation of anybody's in between?

If the answer needs a device, a recipe or a code named before it can be given,
the quantity is outside.

## What happens to a request that falls outside

It is recorded as out of scope, not silently refused and not deleted. A request
that gets no answer comes back, and the second time nobody remembers why it was
turned down the first time.

The record is the request itself, closed with the out-of-scope marking and a
pointer. The pointer names two things and is not optional: which of the four
boundaries above the request fell outside, and where the thing does belong. Where
a project or a register holds it, the pointer names that and gives a resolvable
identifier for it. Where nothing known to this project holds it, the pointer says
so in those words, because "we do not hold this" and "nobody holds this" are
different answers and the second one is useful to whoever asked.

## What this record does not decide

It does not decide whether the numbers a simulation tool ships with are recorded
here. That is a separate question with its own arguments, held elsewhere, and
nothing above answers it: the boundaries here are about what a quantity is about,
and a tool default can fall inside or outside them depending on an answer this
record does not have.

It does not decide the marking vocabulary or the field names, which belong with
the schema.
