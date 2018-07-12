# Incite

Server simulation infrastructure.

# Install

1. Download and install Rustup, the installation tool for the Rust programming language. See [https://www.rust-lang.org]
2. Install the **Rust Nightly** toolchain. This project uses non-stable features which can only be used when compiling with Nightly releases.
3. Download/Clone this repository. `https://github.com/Bert-Proesmans/incite.git`
4. Open a command prompt and change directory to the downloaded sources.
5. Create a new file, called `.env`, for setting required environment data, see below.
6. Execute the command `cargo run --release --bin lobby_server`. This command builds the library after downloading and building its dependancies. When compilation succeeds, the file `/src/bin/lobby_server.rs` is used as main entry point.

## Environment file
The binaries read configuration from an environment file. This file is called `.env` and can be placed next to the binaries or in parent folders.
For re-use purposes and usability it's recommended to create this file at the root of your repository (inside the crate workspace).
This environment file should contain a bash-like syntax, if it can be [source'd](https://ss64.com/bash/source.html) without error you're good to go!
Useful configuration entries are listed below.

- SERVER_ADDRESS
Contains the address and port on which the configured server will bind to. Clients should create a connection with this address.
eg `SERVER_ADDRESS="127.0.0.1:1119"`

- LOG_FILEPATH
The path to the logfile. The log will contain all messages written by the server.
This path can be absolute or relative, in the later it's relative to the 'current working directory'. When running `cargo run [..]` the CWD is equal
to the crate workspace (root of the repository).
eg `LOG_FILEPATH="./server.log"`

# Principles

- (Almost) all components of this system MUST work automatically. Automate all the work! This includes choosing 'sane' defaults for configurable processes.
- (Almost) all code should be written in Rust itself. This includes but isn't limited to build-scripts, setup-scripts, data-provisioning. This limits the amount of components users need to have installed.
- Follow Rust principles regarding code formatting and -structuring. Try to find a readable and understandable mix of explicit- and easy to read procedures.
- Warning are errors!
