#![feature(proc_macro, proc_macro_non_items, generators)]
// TODO; Remove this bandaid.
// Also introduce a #[deny(missing_docs, dead_code)] to keep the code clean.
#![allow(dead_code)]

extern crate bytes;
extern crate chrono;
extern crate dotenv;
pub extern crate incite_gen;
extern crate tokio;
extern crate tokio_codec;
extern crate tokio_signal;
extern crate tokio_tcp;
extern crate tokio_timer;

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

mod log;
pub mod models;
mod protocol;
#[allow(missing_docs)]
pub mod schema;
mod servers;
pub mod setup;

#[derive(Debug)]
pub enum Signal {
    None,
    TermQuit,
}

mod error {
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
        }

        foreign_links {
            Io(io::Error) #[doc = "Error during IO"];
        }
    }
}
