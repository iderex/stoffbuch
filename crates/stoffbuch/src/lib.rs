//! Reads the register.
//!
//! This is the part a consumer depends on. It parses rows, sources, quantities
//! and forms, resolves an identifier at a version, and hands back values. Its
//! audience is another program, so nothing here knows about a terminal, an
//! argument list or an exit code.
//!
//! Empty until the schema exists.

// A deliberate defect, to prove the check on the server goes red for it. It is
// reverted by the next commit on this branch.
fn   badly_spaced ( ) {}
