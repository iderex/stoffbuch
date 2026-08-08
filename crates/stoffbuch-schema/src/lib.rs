//! The schema check.
//!
//! Decides whether a file is a well formed row: its shape, its required
//! fields, its patterns and its canonical serialisation. It is separate from
//! the gate because it is also what a contributor runs against one file, and
//! separate from the library because a consumer reading a released register
//! should not have to carry the machinery that refuses a bad one.
//!
//! Empty until the schema exists.
