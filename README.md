# stoffbuch

Every TCAD, FDTD and device simulation hangs on material parameters that are hard-wired into the tools, differ between tools, and cannot be traced to the measurement they came from. Curated collections exist but none carries versioned provenance, measurement conditions, competing values and an uncertainty. Each row says this value, this source, at this temperature, with this spread between publications, so a simulation can be propagated over the parameter uncertainty rather than over a point value. The work is literature rather than code and builds incrementally.

Planning happens on the issue tracker first. Every decision that shapes
the architecture is written down there with its reasons before the code
that depends on it exists.

See [NOTICE.md](NOTICE.md) for the intended-use notice.

## What these tools send

Nothing. There is no telemetry, no crash reporting, no update check and no
remote lookup that happens without being asked. Everything a run reads it reads
from your disk or from a source you named on the command, and everything it
writes it writes where you asked for it. If a version ever gains a way to publish
your rows to another register, it will be off unless you switch it on and it will
show you the exact bytes it would send first.

If your institution needs to know this before you install anything, the answer in
full, including what a diagnostic you are asked to share may and may not contain,
is in
[docs/decisions/data-protection-and-what-leaves-the-host.md](docs/decisions/data-protection-and-what-leaves-the-host.md).
That file is the one home for it, and no check in this repository yet refuses a
change that would break it.
