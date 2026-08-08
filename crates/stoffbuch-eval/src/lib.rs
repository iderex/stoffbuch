//! The evaluator.
//!
//! Takes a form, a coefficient set and a condition, and produces a value with
//! whatever warnings the domain rules attach to it. It is its own part because
//! the test vectors that hold a form's evaluation contract belong beside it,
//! and because a register that is only being checked never needs to run it.
//!
//! Empty until the form register exists.
