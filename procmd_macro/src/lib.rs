//! This crate is used in the [procmd] crate for the `cmd!` macro. It should not be used directly.
//!
//! [procmd]: https://crates.io/crates/procmd

#![forbid(unsafe_code)]
#![warn(rust_2018_idioms)]

use proc_macro::TokenStream;
use proc_macro2::TokenStream as TokenStream2;
use quote::quote;
use syn::parse::{Parse, ParseStream};
use syn::{parse_macro_input, punctuated::Punctuated, Token};
use vec1::Vec1;

struct Command {
    program: syn::Expr,
    args: Vec<syn::Expr>,
}

impl Parse for Command {
    fn parse(input: ParseStream<'_>) -> syn::Result<Self> {
        let mut exprs =
            Punctuated::<syn::Expr, Token![,]>::parse_separated_nonempty(input)?.into_iter();
        Ok(Command {
            program: exprs.next().unwrap(),
            args: exprs.collect(),
        })
    }
}

struct Commands(Vec1<Command>);

impl Parse for Commands {
    fn parse(input: ParseStream<'_>) -> syn::Result<Self> {
        let mut commands = Vec1::new(Command::parse(input)?);
        while !input.is_empty() {
            <Token![=>]>::parse(input)?;
            commands.push(Command::parse(input)?);
        }
        Ok(Self(commands))
    }
}

impl Commands {
    fn into_token_stream(self) -> TokenStream2 {
        let mut i = 0usize;
        let ts = self.0.mapped_ref(|command| {
            let program = &command.program;
            let args = &command.args;
            i += 1;
            quote! {{
                let mut cmd = ::std::process::Command::new(#program);
                #(cmd.arg(#args);)*
                cmd
            }}
        });
        match ts.split_off_first() {
            (first, rest) if rest.is_empty() => first,
            (first, rest) => {
                let ts = rest.into_iter().fold(first, |mut acc, x| {
                    acc.extend(quote! {,});
                    acc.extend(x);
                    acc
                });
                quote! { ::procmd::PipeCommand::new([#ts]) }
            }
        }
    }
}

#[proc_macro]
pub fn cmd(input: TokenStream) -> TokenStream {
    let commands = parse_macro_input!(input as Commands);
    commands.into_token_stream().into()
}
