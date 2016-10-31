Nom parser for Rust source code
===============================

[![Build Status](https://api.travis-ci.org/dtolnay/syn.svg?branch=master)](https://travis-ci.org/dtolnay/syn)
[![Latest Version](https://img.shields.io/crates/v/syn.svg)](https://crates.io/crates/syn)
[![Rust Documentation](https://img.shields.io/badge/api-rustdoc-blue.svg)](https://dtolnay.github.io/syn/syn/)

Parse Rust structs and enums without a Syntex dependency, intended for use with
[Macros 1.1](https://github.com/rust-lang/rfcs/blob/master/text/1681-macros-1.1.md).

Designed for fast compile time.

- Compile time for `syn` (from scratch including all dependencies): **4 seconds**
- Compile time for the `syntex`/`quasi`/`aster` stack: **60+ seconds**

## Usage with Macros 1.1

```toml
[dependencies]
syn = "0.10.1"
quote = "0.3"

[lib]
proc-macro = true
```

```rust
#![feature(proc_macro, proc_macro_lib)]

extern crate proc_macro;
use proc_macro::TokenStream;

extern crate syn;

#[macro_use]
extern crate quote;

#[proc_macro_derive(MyMacro)]
pub fn my_macro(input: TokenStream) -> TokenStream {
    let source = input.to_string();

    // Parse the string representation to an AST
    let ast = syn::parse_macro_input(&source).unwrap();

    // Build the output, possibly using quasi-quotation
    let expanded = quote! {
        // ...
    };

    // Parse back to a token stream and return it
    expanded.to_string().parse().unwrap()
}
```

## Complete example

Suppose we have the following simple trait which returns the number of fields in
a struct:

```rust
trait NumFields {
    fn num_fields() -> usize;
}
```

A complete Macros 1.1 implementation of `#[derive(NumFields)]` based on `syn`
and [`quote`](https://github.com/dtolnay/quote) looks like this:

```rust
#![feature(proc_macro, proc_macro_lib)]

extern crate proc_macro;
use proc_macro::TokenStream;

extern crate syn;

#[macro_use]
extern crate quote;

#[proc_macro_derive(NumFields)]
pub fn num_fields(input: TokenStream) -> TokenStream {
    let source = input.to_string();

    // Parse the string representation to an AST
    let ast = syn::parse_macro_input(&source).unwrap();

    // Build the output
    let expanded = expand_num_fields(&ast);

    // Return the original input struct unmodified, and the
    // generated impl along with it
    quote!(#ast #expanded).to_string().parse().unwrap()
}

fn expand_num_fields(ast: &syn::MacroInput) -> quote::Tokens {
    let n = match ast.body {
        syn::Body::Struct(ref data) => data.fields().len(),
        syn::Body::Enum(_) => panic!("#[derive(NumFields)] can only be used with structs"),
    };

    // Used in the quasi-quotation below as `#name`
    let name = &ast.ident;

    // Helper is provided for handling complex generic types correctly and effortlessly
    let (impl_generics, ty_generics, where_clause) = ast.generics.split_for_impl();

    quote! {
        // The generated impl
        impl #impl_generics ::mycrate::NumFields for #name #ty_generics #where_clause {
            fn num_fields() -> usize {
                #n
            }
        }
    }
}
```

## Optional features

Syn puts a lot of functionality behind optional features in order to optimize
compile time for the most common use cases. These are the available features and
their effect on compile time. Dependencies are included in the compile times.

Features | Compile time | Functionality
--- | --- | ---
*(none)* | 1 sec | The data structures representing the AST of Rust structs, enums, and types.
parsing | 4 sec | Parsing Rust source code containing structs and enums into an AST.
printing | 2 sec | Printing an AST of structs and enums as Rust source code.
**parsing, printing** | **4 sec** | **This is the default.** Parsing and printing of Rust structs and enums. This is typically what you want for implementing Macros 1.1 custom derives.
full | 2 sec | The data structures representing the full AST of all possible Rust code.
full, parsing | 7 sec | Parsing any valid Rust source code to an AST.
full, printing | 4 sec | Turning an AST into Rust source code.
full, parsing, printing | 8 sec | Parsing and printing any Rust syntax.
full, parsing, printing, expand | 9 sec | Expansion of custom derives in a file of Rust code. This is typically what you want for expanding custom derives on stable Rust using a build script.
full, parsing, printing, expand, pretty | 60 sec | Expansion of custom derives with pretty-printed output. This is what you want when iterating on or debugging a custom derive, but the pretty printing should be disabled once you get everything working.

## Custom derives on stable Rust

Syn supports a way of expanding custom derives from a build script, similar to
what [Serde is able to do with serde_codegen](https://serde.rs/codegen-stable.html).
The advantage of using Syn for this purpose rather than Syntex is much faster
compile time.

Continuing with the `NumFields` example from above, it can be extended to
support stable Rust like this. One or more custom derives are added to a
[`Registry`](https://dtolnay.github.io/syn/syn/struct.Registry.html), which is
then able to expand those derives in a source file at a particular path and
write the expanded output to a different path. A custom derive is represented by
the [`CustomDerive`](https://dtolnay.github.io/syn/syn/trait.CustomDerive.html)
trait which takes a [`MacroInput`](https://dtolnay.github.io/syn/syn/struct.MacroInput.html)
(either a struct or an enum) and [expands it](https://dtolnay.github.io/syn/syn/struct.Expanded.html)
into zero or more new items and maybe a modified or unmodified instance of the
original input.

```rust
pub fn expand_file<S, D>(src: S, dst: D) -> Result<(), String>
    where S: AsRef<Path>,
          D: AsRef<Path>
{
    let mut registry = syn::Registry::new();
    registry.add_derive("NumFields", |input| {
        let tokens = expand_num_fields(&input);
        let items = syn::parse_items(&tokens.to_string()).unwrap();
        Ok(syn::Expanded {
            new_items: items,
            original: Some(input),
        })
    });
    registry.expand_file(src, dst)
}
```

The codegen can be invoked from a build script as follows.

```rust
extern crate your_codegen;

use std::env;
use std::path::Path;

fn main() {
    let out_dir = env::var_os("OUT_DIR").unwrap();

    let src = Path::new("src/codegen_types.in.rs");
    let dst = Path::new(&out_dir).join("codegen_types.rs");

    your_codegen::expand_file(&src, &dst).unwrap();
}
```

## License

Licensed under either of

 * Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
 * MIT license ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.

### Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in this crate by you, as defined in the Apache-2.0 license, shall
be dual licensed as above, without any additional terms or conditions.
