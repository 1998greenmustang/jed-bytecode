use crate::SnakeCase;
use proc_macro::{Delimiter, Ident, TokenStream, TokenTree};

pub fn generate_if_op(iden: &Ident, thing: &Vec<TokenTree>) -> String {
    let first = thing.first();
    match first {
        Some(TokenTree::Group(g)) => {
            assert_eq!(
                g.delimiter(),
                Delimiter::Brace,
                "only blocks have been implemented"
            );
            format!("\"{}\" => {},", &iden.to_snake_case(), g.to_string())
        }
        Some(_) => {
            format!(
                "\"{}\" => Operation::{}({}),",
                &iden.to_snake_case(),
                iden.to_string(),
                TokenStream::from_iter(thing.iter().map(|x| x.clone())).to_string()
            )
        }
        None => {
            format!(
                "\"{}\" => Operation::{},",
                &iden.to_snake_case(),
                iden.to_string(),
            )
        }
    }
}
