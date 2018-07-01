# Incite

Server simulation infrastructure.

# Install

1. Download and install Rustup, the installation tool for the Rust programming language. See [https://www.rust-lang.org]
2. Install the **Rust Nightly** toolchain. This project uses non-stable features which can only be used when compiling with Nightly releases.
3. Download/Clone this repository. `https://github.com/Bert-Proesmans/incite.git`
4. Open a command prompt and change directory to the downloaded sources.
5. Execute the command `cargo run --release --bin lobby_server`. This command builds the library after downloading and building its dependancies. When compilation succeeds, the file `/src/bin/lobby_server.rs` is used as main entry point.

# Principles

- (Almost) all components of this system MUST work automatically. Automate all the work! This includes choosing 'sane' defaults for configurable processes.
- (Almost) all code should be written in Rust itself. This includes but isn't limited to build-scripts, setup-scripts, data-provisioning. This limits the amount of components users need to have installed.
- Follow Rust principles regarding code formatting and -structuring. Try to find a readable and understandable mix of explicit- and easy to read procedures.
- Warning are errors!
