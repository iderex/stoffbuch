Id: no-default-for-an-absent-condition
Held: yes
Subject: crates/
Spelling: room_temperature
Spelling: roomtemperature
Spelling: default_temperature
Spelling: defaulttemperature
Spelling: default_condition
Spelling: defaultcondition
Spelling: assumed_temperature
Spelling: assumedtemperature
Spelling: assume_room
Retired-by: nothing yet. It is retired if the condition block is ever removed from the register, which would remove what it guards.
Rule: a condition is never supplied by code. A condition the publication did not state is recorded as not stated, and no default, no assumed room temperature and no helper that fills one in may exist.

`docs/decisions/quantities-units-and-conditions.md` decides this and states the
reason: room temperature is itself a range that different publications mean
differently, and filling it in is precisely how the untraceable numbers this
project is a reaction to were made. A row whose temperature was never published
carries `not-stated`, which is a value a curator writes rather than an empty
field, and a report prints it as a sentence saying the publication does not
state a temperature.

The failure this prevents is a convenience. The helper that supplies a plausible
temperature so that an evaluation can proceed is a small and reasonable-looking
change, it is the change somebody makes under time pressure, and once it exists
every row that was honest about not knowing becomes a row that quietly knows.
Nothing downstream can tell the two apart afterwards, because the invented value
looks exactly like a published one.

The condition block does not exist in the tree yet, so this invariant refuses
nothing today. That is why it is here now. A guard that arrives after the code
it guards is a guard that has to be argued against work somebody has already
done, and this one is cheap enough to stand before there is anything for it to
catch.

## What it does not catch

A number written as a bare literal. `298.15` in source is the same defect and it
is not listed, because a temperature in a conversion test vector is legitimate
work written in exactly that form, and an invariant that refuses honest work is
one somebody weakens until it holds nothing.

A default supplied under a name nobody thought of. The spellings are the ones a
person writing this mistake would reach for, and a spelling outside the list
walks through. What answers that is the review, and the list growing the day a
new spelling is seen.

A default that is not about temperature. The condition block is not only
temperature, and the spellings here are, because temperature is the condition
the decision record names and the one a convenience helper would supply first.
The rule above is wider than the spellings that hold it, deliberately, so that
the record says what is meant rather than what is matched.
