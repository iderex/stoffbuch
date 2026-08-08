//! Reads the register.
//!
//! This is the part a consumer depends on. It parses rows, sources, quantities
//! and forms, resolves an identifier at a version, and hands back values. Its
//! audience is another program, so nothing here knows about a terminal, an
//! argument list or an exit code.
//!
//! Empty until the schema exists.
