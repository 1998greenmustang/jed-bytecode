use std::{collections::HashMap, str::FromStr};

use proc_macro::{Ident, TokenStream, TokenTree};

use crate::ast::{Ast, AstEntry};

pub fn enum_exists<T: Fn(&String) -> String>(ast: Ast, name_fn: T) -> TokenStream {
    let mut eq_stmts: Vec<String> = Vec::new();
    if let Some(AstEntry::Enum(name, ast)) = ast.first() {
        for node in ast {
            match node {
                AstEntry::Variant(vname, _tpl) => {
                    eq_stmts.push(format!("txt == \"{}\"", name_fn(vname)))
                }
                _ => {}
            }
        }
        let output = &format!(
            // impl {name} { pub fn exists(txt: &str) -> bool { {} }}}",
            "impl {} {{\n\tpub fn exists(txt: &str) -> bool {{\n\t\t{}\n\t}}\n}}\n",
            name.to_string(),
            eq_stmts.join(" || ")
        );
        return TokenStream::from_str(&output).expect("yea man");
    } else {
        todo!()
    }
}
