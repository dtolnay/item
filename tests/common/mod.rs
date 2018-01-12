// Copyright 2018 Syn Developers
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

#![allow(dead_code)]

extern crate syntax;
extern crate walkdir;

use std;
use std::env;
use std::process::Command;

use self::walkdir::DirEntry;

pub mod parse;
pub mod respan;

pub fn check_min_stack() {
    let min_stack_value = env::var("RUST_MIN_STACK")
        .expect("RUST_MIN_STACK env var should be set since some tests require it.");
    let min_stack_value: usize = min_stack_value
        .parse()
        .expect("RUST_MIN_STACK env var should be set since some tests require it.");
    assert!(min_stack_value >= 16_000_000);
}

/// Read the `ABORT_AFTER_FAILURE` environment variable, and parse it.
pub fn abort_after() -> usize {
    match env::var("ABORT_AFTER_FAILURE") {
        Ok(s) => s.parse().expect("failed to parse ABORT_AFTER_FAILURE"),
        Err(_) => std::usize::MAX,
    }
}

pub fn base_dir_filter(entry: &DirEntry) -> bool {
    let path = entry.path();
    if path.is_dir() {
        return true; // otherwise walkdir does not visit the files
    }
    if path.extension().map(|e| e != "rs").unwrap_or(true) {
        return false;
    }
    let path_string = path.to_string_lossy();
    let path_string = if cfg!(windows) {
        path_string.replace('\\', "/").into()
    } else {
        path_string
    };
    // TODO assert that parsing fails on the parse-fail cases
    if path_string.starts_with("tests/rust/src/test/parse-fail")
        || path_string.starts_with("tests/rust/src/test/compile-fail")
    {
        return false;
    }

    if path_string.starts_with("tests/rust/src/test/ui") {
        let stderr_path = path.with_extension("stderr");
        if stderr_path.exists() {
            // Expected to fail in some way
            return false;
        }
    }

    match path_string.as_ref() {
        // TODO better support for attributes
        //
        //      let a = A { #[a] b: c };
        "tests/rust/src/librustc_data_structures/blake2b.rs" |
        // TODO better support for attributes
        //
        //      enum A { B = #[a] 2 }
        "tests/rust/src/test/incremental/hashes/enum_defs.rs" |
        // TODO better support for attributes
        //
        //      { #![foo] }
        "tests/rust/src/test/pretty/stmt_expr_attributes.rs" |
        // TODO better support for attributes
        "tests/rust/src/test/run-pass/cfg_stmt_expr.rs" |
        // TODO weird glob import
        //
        //      use ::*;
        "tests/rust/src/test/run-pass/import-glob-crate.rs" |
        // TODO better support for attributes
        //
        //      impl Foo { #![a] }
        "tests/rust/src/test/run-pass/inner-attrs-on-impl.rs" |
        // TODO better support for attributes
        "tests/rust/src/test/run-pass/item-attributes.rs" |
        // TODO support lifetimes before traits
        //
        //      Box<'foo + Bar>
        "tests/rust/src/test/run-pass/trait-object-lifetime-first.rs" |
        // TODO feature(use_nested_groups)
        //
        //      use a::{B, D::{self, *, g::H}};
        "tests/rust/src/test/run-pass/use-nested-groups.rs" |
        // Deprecated placement syntax
        "tests/rust/src/test/run-pass/new-box-syntax.rs" |
        // Deprecated placement syntax
        "tests/rust/src/test/run-pass/placement-in-syntax.rs" |
        // not actually a test case
        "tests/rust/src/test/run-pass/auxiliary/macro-include-items-expr.rs" => false,
        _ => true,
    }
}

pub fn clone_rust() {
    let result = Command::new("tests/clone.sh").status().unwrap();
    println!("result: {}", result);
    assert!(result.success());
}
