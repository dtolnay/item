//! Extension traits to provide parsing methods on foreign types.
//!
//! *This module is available if Syn is built with the `"parsing"` feature.*

use proc_macro2::Ident;

use parse::{ParseStream, Result};
use punctuated::Punctuated;
#[cfg(any(feature = "full", feature = "derive"))]
use path::{Path, PathSegment};

/// Additional parsing methods for `Ident`.
///
/// This trait is sealed and cannot be implemented for types outside of Syn.
///
/// *This trait is available if Syn is built with the `"parsing"` feature.*
pub trait IdentExt: Sized + private::Sealed {
    /// Parses any identifier including keywords.
    ///
    /// This is useful when parsing a DSL which allows Rust keywords as
    /// identifiers.
    ///
    /// ```edition2018
    /// use syn::{Error, Ident, Result, Token};
    /// use syn::ext::IdentExt;
    /// use syn::parse::ParseStream;
    ///
    /// // Parses input that looks like `name = NAME` where `NAME` can be
    /// // any identifier.
    /// //
    /// // Examples:
    /// //
    /// //     name = anything
    /// //     name = impl
    /// fn parse_dsl(input: ParseStream) -> Result<Ident> {
    ///     let name_token: Ident = input.parse()?;
    ///     if name_token != "name" {
    ///         return Err(Error::new(name_token.span(), "expected `name`"));
    ///     }
    ///     input.parse::<Token![=]>()?;
    ///     let name = input.call(Ident::parse_any)?;
    ///     Ok(name)
    /// }
    /// ```
    fn parse_any(input: ParseStream) -> Result<Self>;
    /// Peeks for any identifier including keywords.
    fn peek_any(input: ParseStream) -> bool;
}

impl IdentExt for Ident {
    fn parse_any(input: ParseStream) -> Result<Self> {
        input.step(|cursor| match cursor.ident() {
            Some((ident, rest)) => Ok((ident, rest)),
            None => Err(cursor.error("expected ident")),
        })
    }

    fn peek_any(input: ParseStream) -> bool {
        let ahead = input.fork();
        Self::parse_any(&ahead).is_ok()
    }
}

#[cfg(any(feature = "full", feature = "derive"))]
/// Additional parsing methods for `Path`.
///
/// This trait is sealed and cannot be implemented for types outside of Syn.
///
/// *This trait is available if Syn is built with the `"parsing"` feature.*
pub trait PathExt: Sized + private::Sealed {
    /// Parse a `Path` in mod style, while accepting keywords.
    ///
    /// This function is only available within `syn` to parse meta items.
    ///
    /// *This function is available if Syn is built with the `"parsing"`
    /// feature.*
    fn parse_meta(input: ParseStream) -> Result<Self>;
}

#[cfg(any(feature = "full", feature = "derive"))]
impl PathExt for Path {
    fn parse_meta(input: ParseStream) -> Result<Self> {
        Ok(Path {
            leading_colon: input.parse()?,
            segments: {
                let mut segments = Punctuated::new();
                loop {
                    if !Ident::peek_any(input) {
                        break;
                    }
                    let ident = Ident::parse_any(input)?;
                    segments.push_value(PathSegment::from(ident));
                    if !input.peek(Token![::]) {
                        break;
                    }
                    let punct = input.parse()?;
                    segments.push_punct(punct);
                }
                if segments.is_empty() {
                    return Err(input.error("expected path"));
                } else if segments.trailing_punct() {
                    return Err(input.error("expected path segment"));
                }
                segments
            },
        })
    }
}

mod private {
    use proc_macro2::Ident;

    #[cfg(any(feature = "full", feature = "derive"))]
    use path::Path;

    pub trait Sealed {}

    impl Sealed for Ident {}

    #[cfg(any(feature = "full", feature = "derive"))]
    impl Sealed for Path {}
}
