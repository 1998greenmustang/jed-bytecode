use std::iter::Peekable;

use proc_macro::{Delimiter, Ident, TokenStream, TokenTree, token_stream::IntoIter};

// i want to test if its actually a 'kind'able identifier
// but, that can be pretty complex i dont really want to deal with
// so this just lets the rustc to check (meaning no good error message)
pub fn is_kind(token: &TokenTree) -> bool {
    match token {
        TokenTree::Group(_) | TokenTree::Ident(_) => true,
        _ => false,
    }
}

pub fn tuple_count(kind: &String) -> usize {
    kind.chars().filter(|&c| c == ',').count() + 1
}

pub trait SnakeCase<T: ToString = Self>: ToString {
    fn to_snake_case(&self) -> String {
        let mut snake_case = String::new();
        let txt = self.to_string();
        let iter = txt.char_indices().into_iter();
        let iter1 = txt.char_indices().into_iter();

        for ((i, c), (i1, c1)) in iter.zip(iter1.skip(1)) {
            if c.is_ascii_uppercase() && i != 0 && (c1.is_ascii_lowercase() || c1.is_ascii_digit())
            {
                snake_case.push('_');
            }
            snake_case.push(c.to_ascii_lowercase());
            if i1 == txt.len() - 1 {
                snake_case.push(c1.to_ascii_lowercase());
            }
        }
        return snake_case;
    }
}

impl SnakeCase<&String> for &String {}

impl SnakeCase<Ident> for Ident {}

pub fn empty_tuple(kind: &String) -> String {
    if kind == "()" {
        "".to_owned()
    } else {
        let underscores: Vec<&str> = (0..tuple_count(kind)).map(|_| "_").collect();
        underscores.join(",")
    }
}

pub fn get_deconstructor(kind: &String) -> String {
    if kind == "()" {
        "".to_owned()
    } else {
        let args: Vec<String> = (0..tuple_count(kind))
            .map(|i| "a".to_owned() + &i.to_string())
            .collect();
        "(".to_owned() + &args.join(",") + ")"
    }
}

pub fn is_punct(token: &TokenTree, punctor: char) -> bool {
    match token {
        TokenTree::Punct(p) => p.as_char() == punctor,
        _ => false,
    }
}

pub fn is_group(token: &TokenTree, delimiter: Delimiter) -> bool {
    match token {
        TokenTree::Group(g) => g.delimiter() == delimiter,
        _ => false,
    }
}

pub fn is_ident(token: &TokenTree) -> bool {
    match token {
        TokenTree::Ident(_) => true,
        _ => false,
    }
}

pub fn collect_idents(token_iter: &mut Peekable<IntoIter>) -> Vec<Ident> {
    let mut ids: Vec<Ident> = vec![];
    if let Some(TokenTree::Ident(i)) = token_iter.next_if(|x| is_ident(x)) {
        ids.push(i);
    }
    while let Some(_) = token_iter.next_if(|x| is_punct(x, ',')) {
        if let Some(TokenTree::Ident(i)) = token_iter.next_if(|x| is_ident(x)) {
            ids.push(i);
        }
    }
    ids
}
