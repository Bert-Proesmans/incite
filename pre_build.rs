// build.rs
extern crate vergen;

use vergen::{vergen, OutputFns};

fn main() {
    // Select all available commit data to export from git.
    let version_flags = OutputFns::all();
    assert!(vergen(version_flags).is_ok());
}
