use std::{collections::HashMap, str::FromStr};

use proc_macro::TokenStream;

use crate::ast::{Ast, AstEntry};

pub fn index_to_name<T: Fn(&String) -> String>(ast: Ast, name_fn: T) -> TokenStream {
    let mut idx_stmts: Vec<String> = Vec::new();
    if let Some(AstEntry::Enum(name, ast)) = ast.first() {
        let variants = ast.iter().filter(|x| matches!(x, AstEntry::Variant(_, _)));
        for (i, v) in variants.enumerate() {
            if let AstEntry::Variant(vname, _tpl) = v {
                idx_stmts.push(format!("{i} => \"{}\"", name_fn(vname)))
            }
        }
        let output = &format!(
            // impl {name} { pub fn from_index(idx: &u8) -> String { match idx { {} } }}}",
            "
impl {} {{
    pub fn from_index(idx: &u8) -> &str {{
        match idx {{
            {},
            _ => todo!(\"{{idx}}\")
        }}
    }}
}}
            ",
            name.to_string(),
            idx_stmts.join(",\n\t    ")
        );
        return TokenStream::from_str(&output).expect("yea man");
    } else {
        todo!()
    }
}

pub fn name_to_index<T: Fn(&String) -> String>(ast: Ast, name_fn: T) -> TokenStream {
    let mut idx_stmts: Vec<String> = Vec::new();
    if let Some(AstEntry::Enum(name, ast)) = ast.first() {
        let variants = ast.iter().filter(|x| matches!(x, AstEntry::Variant(_, _)));
        for (i, v) in variants.enumerate() {
            if let AstEntry::Variant(vname, _tpl) = v {
                idx_stmts.push(format!("\"{}\" => {i}", name_fn(vname)))
            }
        }
        let output = &format!(
            "
impl {} {{
    pub fn to_index(txt: &str) -> u8 {{
        match txt {{
            {},
            _ => todo!(\"{{txt}}\")
        }}
    }}
}}
            ",
            name.to_string(),
            idx_stmts.join(",\n\t    ")
        );
        return TokenStream::from_str(&output).expect("yea man");
    } else {
        todo!()
    }
}

type IndexedVariant = (usize, String);

struct FromStructure {
    constructor: String,
    variants: Vec<IndexedVariant>,
}

// match value.0 {
//     2 => EnumName::Variant(value.1.0, value.1.1)
// }

// variant inde
pub fn index_froms(ast: Ast) -> TokenStream {
    let mut kinds: HashMap<String, FromStructure> = HashMap::new();
    let mut into_index_stmts: Vec<String> = Vec::new();
    if let Some(AstEntry::Enum(name, ast)) = ast.first() {
        let variants = ast.iter().filter(|x| matches!(x, AstEntry::Variant(_, _)));
        for (i, v) in variants.enumerate() {
            if let AstEntry::Variant(vname, tpl) = v {
                // into_index
                let deconstruct = if tpl.len() == 0 {
                    "".to_owned()
                } else {
                    format!("({})", "_,".repeat(tpl.len()))
                };
                into_index_stmts.push(format!("{name}::{vname}{} => {i}", deconstruct));

                // sort into kind buckets
                let kind = if tpl.len() == 0 {
                    "".to_owned()
                } else if tpl.len() == 1 {
                    tpl.get(0).unwrap().to_string()
                } else {
                    "(".to_owned()
                        + &tpl
                            .iter()
                            .map(|x| x.to_string())
                            .collect::<Vec<String>>()
                            .join(", ")
                        + ")"
                };
                if let Some(FromStructure {
                    constructor: _,
                    variants: vs,
                }) = kinds.get_mut(&kind)
                {
                    vs.push((i, vname.clone()));
                } else {
                    let constructor = if tpl.len() == 0 {
                        "".to_owned()
                    } else if tpl.len() == 1 {
                        "(value.1)".to_owned()
                    } else {
                        format!(
                            "({})",
                            (0..tpl.len())
                                .map(|i| format!("value.1.{}", i))
                                .collect::<Vec<String>>()
                                .join(",")
                        )
                    };
                    kinds.insert(
                        kind.clone(),
                        FromStructure {
                            constructor,
                            variants: vec![(i, vname.clone())],
                        },
                    );
                }
            }
        }
        let mut output_impls: Vec<String> = Vec::new();
        output_impls.push(format!(
            "
impl From<{name}> for u8 {{
    fn from(value: {name}) -> Self {{
            match value {{{}, _ => panic!()}}
    }}
}}",
            into_index_stmts.join(",\n")
        ));

        for (k, s) in kinds.iter() {
            if k == "" {
                output_impls.push(format!(
                    "
impl From<u8> for {name} {{
    fn from(value: u8) -> Self {{
            match value {{{}, _ => panic!()}}
    }}
}}",
                    s.variants
                        .iter()
                        .map(|x| format!("{} => {name}::{}{}", x.0, x.1, s.constructor))
                        .collect::<Vec<String>>()
                        .join(",\n")
                ));
            } else {
                let from_kind = format!("(u8, {})", k);
                output_impls.push(format!(
                    "
impl From<{from_kind}> for {name} {{
    fn from(value: {from_kind}) -> Self {{
            match value.0 {{{}, _ => panic!()}}
    }}
}}",
                    s.variants
                        .iter()
                        .map(|x| format!("{} => {name}::{}{}", x.0, x.1, s.constructor))
                        .collect::<Vec<String>>()
                        .join(",\n")
                ));
            }
        }
        return TokenStream::from_str(&output_impls.join("\n")).expect("yea dude");
    } else {
        todo!()
    }
}
