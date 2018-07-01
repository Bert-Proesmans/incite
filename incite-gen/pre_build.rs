#![feature(nll)]

extern crate glob;
extern crate prost_build;
extern crate vergen;

use glob::glob;
use std::env;
use std::ffi::OsString;
use std::fs::{DirBuilder, File};
use std::io;
use std::io::prelude::*;
use std::path::PathBuf;
use vergen::{vergen, OutputFns};

fn main() {
    // Select all available commit data to export from git.
    let version_flags = OutputFns::all();
    assert!(vergen(version_flags).is_ok());

    let proto_out_dir = build_protos();
    concat_protos(proto_out_dir);
}

fn build_protos() -> OsString {
    let mut source_dir = PathBuf::new();
    source_dir.push(env::var("CARGO_MANIFEST_DIR").unwrap());
    source_dir.push("..");
    source_dir.push("proto");
    // let source_dir = source_dir.canonicalize().expect("proto input");
    let source_dir = source_dir.to_str().unwrap();

    println!("Pulling proto files from {}", source_dir);
    let proto_files_glob = format!("{:}/**/*.proto", source_dir);
    let proto_paths: Vec<_> = glob(&proto_files_glob)
        .unwrap()
        .filter_map(Result::ok)
        .collect();
    let proto_files: Vec<_> = proto_paths
        .iter()
        .filter_map(|path| path.to_str())
        .collect();

    // Compiled proto schemas are located in OUT_DIR.
    // This variable is overwritten to redirect the proto output.
    let old_out_dir = env::var_os("OUT_DIR").unwrap();
    //
    let mut proto_out_dir = old_out_dir.clone();
    proto_out_dir.push("proto_out");
    let proto_out_dir = proto_out_dir;
    DirBuilder::new()
        .recursive(true)
        .create(proto_out_dir.clone())
        .expect("proto-out folder");
    env::set_var("OUT_DIR", &proto_out_dir);
    //
    prost_build::compile_protos(&proto_files[..], &[source_dir]).expect("protoc");
    // Restore the old OUT_DIR variable
    env::set_var("OUT_DIR", &old_out_dir);

    // Only execute this script again if the proto source directory has changed.
    println!("cargo:rerun-if-changed={}", source_dir);

    proto_out_dir
}

fn concat_protos(proto_directory: OsString) {
    let proto_files_glob = format!("{:}/**/*.rs", proto_directory.to_str().unwrap());
    let mut generated_proto_filenames: Vec<_> = glob(&proto_files_glob)
        .unwrap()
        .filter_map(Result::ok)
        .collect();
    generated_proto_filenames.sort_unstable();

    let mut target_file = PathBuf::new();
    target_file.push(env::var_os("OUT_DIR").unwrap());
    target_file.push("combined_generated_proto.rs");
    let target_file = File::create(target_file).expect("combined proto");
    let mut target_file_writer = io::BufWriter::new(target_file);

    // Precondition: The modules must be sorted before combining them into the same file.
    //
    // - Rust disallows multiple declarations of the same module.
    let mut current_module: Vec<String> = vec![];
    while let Some(file_path) = generated_proto_filenames.pop() {
        println!("Processing proto file {}", file_path.to_str().unwrap());
        let compiled_file = File::open(file_path.clone()).expect("proto open");
        let mut file_reader = io::BufReader::new(compiled_file);
        let module_parts: Vec<_> = file_path
            .file_stem()
            .unwrap()
            .to_str()
            .unwrap()
            .split(".")
            .collect();

        let mut current_idx = 0;
        'compiled: loop {
            let current_part = match module_parts.get(current_idx) {
                Some(item) => item,
                None => {
                    // current_module holds exactly our module parts WITH OPTIONALLY more parts.
                    // This happens when going from a.b.c -> a.b
                    let closed_modules = current_module.split_off(current_idx);
                    for _ in closed_modules {
                        let module_block_end = "}\n".as_bytes();
                        target_file_writer
                            .write(module_block_end)
                            .expect("proto write");
                    }
                    io::copy(&mut file_reader, &mut target_file_writer)
                        .expect("Proto concat write");
                    break;
                }
            };

            'current: loop {
                match current_module.get(current_idx) {
                    Some(ref mut current) if current == current_part => {}
                    Some(_) => {
                        // Non matching module part, close these modules to synchronize
                        // with our module parts.
                        let closed_modules = current_module.split_off(current_idx);
                        for _ in closed_modules {
                            let module_block_end = "}\n".as_bytes();
                            target_file_writer
                                .write(module_block_end)
                                .expect("proto write");
                        }
                        continue 'current;
                    }
                    None => {
                        // Our own module part is not opened, so we do that here and record
                        // this operation.
                        let module_block_start =
                            format!("pub mod {:} {{\n", current_part).into_bytes();
                        target_file_writer
                            .write(&module_block_start)
                            .expect("proto write");
                        current_module.push(String::from(*current_part));
                    }
                }

                current_idx = current_idx + 1;
                continue 'compiled;
            }
        }
    }

    // We must clean up because the algorithm leaves the module hierarchy in an open state.
    for _ in current_module {
        let module_block_end = "}\n".as_bytes();
        target_file_writer
            .write(module_block_end)
            .expect("proto write");
    }
}
