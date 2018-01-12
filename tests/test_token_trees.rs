// Copyright 2018 Syn Developers
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

#![cfg(feature = "extra-traits")]

extern crate proc_macro2;
#[macro_use]
extern crate quote;
extern crate syn;

use syn::{AttrStyle, Attribute, Lit};
use proc_macro2::{Delimiter, Spacing, Span, Term, TokenNode, TokenStream, TokenTree};
use proc_macro2::Delimiter::*;

fn alone(c: char) -> TokenTree {
    proc_macro2::TokenTree {
        span: Span::def_site(),
        kind: TokenNode::Op(c, Spacing::Alone),
    }
}

fn joint(c: char) -> TokenTree {
    proc_macro2::TokenTree {
        span: Span::def_site(),
        kind: TokenNode::Op(c, Spacing::Joint),
    }
}

fn delimited(delim: Delimiter, tokens: Vec<TokenTree>) -> TokenTree {
    proc_macro2::TokenTree {
        span: Span::def_site(),
        kind: TokenNode::Group(delim, tokens.into_iter().collect()),
    }
}

fn word(sym: &str) -> TokenTree {
    proc_macro2::TokenTree {
        span: Span::def_site(),
        kind: TokenNode::Term(Term::intern(sym)),
    }
}

#[test]
fn test_struct() {
    let raw = "
        #[derive(Debug, Clone)]
        pub struct Item {
            pub ident: Ident,
            pub attrs: Vec<Attribute>,
        }
    ";

    let expected = vec![
        alone('#'),
        delimited(
            Bracket,
            vec![
                word("derive"),
                delimited(Parenthesis, vec![word("Debug"), alone(','), word("Clone")]),
            ],
        ),
        word("pub"),
        word("struct"),
        word("Item"),
        delimited(
            Brace,
            vec![
                word("pub"),
                word("ident"),
                alone(':'),
                word("Ident"),
                alone(','),
                word("pub"),
                word("attrs"),
                alone(':'),
                word("Vec"),
                alone('<'),
                word("Attribute"),
                joint('>'),
                alone(','),
            ],
        ),
    ];

    fn wrap(tts: TokenStream) -> Attribute {
        Attribute {
            style: AttrStyle::Outer,
            pound_token: Default::default(),
            bracket_token: Default::default(),
            path: "test".into(),
            tts: tts,
            is_sugared_doc: false,
        }
    }

    let result = wrap(raw.parse().unwrap());
    let expected = wrap(expected.into_iter().collect());
    if result != expected {
        panic!("{:#?}\n!=\n{:#?}", result.tts, expected.tts);
    }
}

#[test]
fn test_literal_mangling() {
    let raw = "0_4";
    let parsed: Lit = syn::parse_str(raw).unwrap();
    assert_eq!(raw, quote!(#parsed).to_string());
}
