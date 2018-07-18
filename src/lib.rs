#![feature(proc_macro_non_items, generators, use_extern_macros)]
// TODO; Remove this bandaid.
// Also introduce a #[deny(missing_docs, dead_code)] to keep the code clean.
#![allow(dead_code)]
// TODO: Remove the following attribute.
// Diesel dependancy needs to be updated because it performs imports inside its macros without explicit
// path. See https://github.com/rust-lang/rust/issues/50504.
// This flag is allowed to reduce terminal noise while compiling.
#![allow(proc_macro_derive_resolution_fallback)]

extern crate bytes;
extern crate chrono;
extern crate dotenv;
extern crate failure;
pub extern crate incite_gen;
extern crate tokio;
extern crate tokio_codec;
extern crate tokio_signal;
extern crate tokio_tcp;
extern crate tokio_timer;

#[macro_use]
extern crate lazy_static;
#[macro_use]
extern crate maplit;
// #[macro_use]
extern crate futures_await as futures;
#[macro_use]
extern crate typed_builder;
#[macro_use]
extern crate slog;
#[macro_use]
extern crate error_chain;
#[macro_use]
extern crate diesel;

pub mod log;
pub mod models;
pub mod protocol;
#[allow(missing_docs)]
pub mod schema;
pub mod servers;
pub mod service;
pub mod services;
pub mod setup;

#[derive(Debug)]
pub enum Signal {
    None,
    TermQuit,
}

pub mod error {
    use super::*;
    use std::io;

    error_chain! {
        errors {
            Interrupted(s: Signal) {
                description("The user interrupted by sending a signal")
                display("User interrupted program.")
            }
        }

        links {
            Setup(setup::Error, setup::ErrorKind);
            Lobby(servers::lobby::Error, servers::lobby::ErrorKind);
        }

        foreign_links {
            Io(io::Error) #[doc = "Error during IO"];
        }
    }
}
pub use self::error::*;
