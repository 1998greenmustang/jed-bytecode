extern crate proc_macro;
mod ast;
mod generators;
mod macros;
mod utils;
use std::str::FromStr;
use utils::SnakeCase;

use proc_macro::*;

use generators::*;
use utils::*;

use crate::macros::{enum_display::*, enum_exists::*, index};

// #[derive(SnakeCaseDisplay)]
// enum Test {
//     #[jed(type: Option<usize>, func: option_display )]
//     TestEntry,
// }
#[proc_macro_derive(SnakeCaseDisplay, attributes(jed))]
pub fn snake_case_enum_display(stream: TokenStream) -> TokenStream {
    match ast::parse(stream) {
        Ok(ast) => enum_display(ast, |x: &String| x.to_snake_case()),
        Err(e) => panic!("{e:?}"),
    }
}

#[proc_macro_derive(IndexToSnakeCase, attributes(jed))]
pub fn index_to_snake_case(stream: TokenStream) -> TokenStream {
    match ast::parse(stream) {
        Ok(ast) => index::index_to_name(ast, |x: &String| x.to_snake_case()),
        Err(e) => panic!("{e:?}"),
    }
}

#[proc_macro_derive(SnakeCaseToIndex, attributes(jed))]
pub fn snake_case_to_index(stream: TokenStream) -> TokenStream {
    match ast::parse(stream) {
        Ok(ast) => index::name_to_index(ast, |x: &String| x.to_snake_case()),
        Err(e) => panic!("{e:?}"),
    }
}

//
#[proc_macro_derive(IndexFroms, attributes(jed))]
pub fn index_froms(stream: TokenStream) -> TokenStream {
    match ast::parse(stream) {
        Ok(ast) => index::index_froms(ast),
        Err(e) => panic!("{e:?}"),
    }
}

#[proc_macro_derive(SnakeCaseExists, attributes(jed))]
pub fn snake_case_enum_exists(stream: TokenStream) -> TokenStream {
    match ast::parse(stream) {
        Ok(ast) => enum_exists(ast, |x: &String| x.to_snake_case()),
        Err(e) => panic!("{e:?}"),
    }
}

#[proc_macro]
pub fn match_ops(stream: TokenStream) -> TokenStream {
    // register_op!(Ident or [Ident,...], Expr or Block)
    let mut iter = stream.into_iter().peekable();
    let mut outputs: Vec<String> = vec!["match op {".to_owned()];
    while let Some(TokenTree::Group(grp)) = iter.next_if(|x| is_group(x, Delimiter::Brace)) {
        let mut g_iter = grp.stream().into_iter().peekable();
        let first = g_iter.next();
        let _ = g_iter.next_if(|x| is_punct(x, ','));
        let second = g_iter.collect::<Vec<TokenTree>>();

        outputs.push(match first {
            Some(TokenTree::Group(g)) => {
                assert_eq!(g.delimiter(), Delimiter::Bracket);
                let mut g_iter = g.stream().into_iter().peekable();
                let ids = collect_idents(&mut g_iter);
                ids.iter()
                    .map(|i| generate_if_op(i, &second))
                    .collect::<Vec<String>>()
                    .join("\n")
            }
            Some(TokenTree::Ident(i)) => generate_if_op(&i, &second),
            _ => panic!("give me smth brother"),
        });
        let _ = iter.next_if(|x| is_punct(x, ','));
    }
    outputs.push("_ => todo!(\"{}\", op)}".to_owned());
    return TokenStream::from_str(&outputs.join("\n")).expect("comon");
}
