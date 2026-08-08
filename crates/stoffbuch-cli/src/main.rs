//! The command line.
//!
//! The only part of this workspace with an audience that is a person, and so
//! the only one allowed to know about a terminal, an argument list or an exit
//! code. Everything it does it does by calling one of the other crates, so a
//! behaviour that exists only here is a behaviour no other consumer can reach.
//!
//! Empty until there is something to run.

fn main() {}
