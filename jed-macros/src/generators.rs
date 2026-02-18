use std::{collections::HashMap, str::FromStr};

use proc_macro::{Delimiter, Ident, TokenStream, TokenTree};

use crate::{
    KindStuff, OperationDefinition, empty_tuple, get_deconstructor, snake_case, tuple_count,
};

pub fn generate_enum(slice: &[OperationDefinition]) -> TokenStream {
    let mut ops_as_strings: Vec<String> = vec![];
    let mut emptyop_as_strings: Vec<String> = vec![];
    for op in slice {
        let op_str = op.iden.to_string();
        if op.kind.to_string() == "()" {
            emptyop_as_strings.push(format!("{}", op_str.clone(),));
            ops_as_strings.push(op_str);
        } else {
            emptyop_as_strings.push(format!(
                "{}({})",
                op_str.clone(),
                empty_tuple(&op.kind.to_string())
            ));
            ops_as_strings.push(format!("{}{}", op_str, op.kind.to_string()));
        }
    }
    let enum_str = format!(
        "
#[derive(Copy, Clone, Debug)]
pub enum Operation {{{}}}",
        ops_as_strings.join(",")
    );

    let to_byte = format!(
        "
impl From<Operation> for u8 {{
    fn from(value: Operation) -> Self {{
        match value {{{}, _ => panic!()}}
    }}
}}
",
        emptyop_as_strings
            .iter()
            .enumerate()
            .map(|(i, s)| format!("Operation::{s} => {i}"))
            .collect::<Vec<String>>()
            .join(",")
    );

    TokenStream::from_str(&(enum_str + "\n" + &to_byte))
        .expect("parsing error bro; must be an invalid type")
}

pub fn generate_kind_things(kinds: HashMap<String, KindStuff>) -> TokenStream {
    let mut output: Vec<String> = vec![];
    let mut formatting_stuff: Vec<String> = vec![];
    for (k, v) in kinds.iter() {
        // make the formatting things
        for (idx, op) in v.operations.clone() {
            if k == "()" {
                formatting_stuff.push(format!(
                    "Operation::{} => write!(f, \"{}\")",
                    op.iden.to_string(),
                    op.snake_case,
                ));
            } else {
                let deconstruct = get_deconstructor(k);
                match &v.display_func {
                    Some(f) => {
                        formatting_stuff.push(format!(
                            "Operation::{}{deconstruct} => write!(f, \"{} {{}}\", {}{deconstruct})",
                            op.iden.to_string(),
                            op.snake_case,
                            f
                        ));
                    }
                    None => {
                        let brackets_needed = tuple_count(k);
                        formatting_stuff.push(format!(
                            "Operation::{}{deconstruct} => write!(f, \"{} {}\", {})",
                            op.iden.to_string(),
                            op.snake_case,
                            (0..brackets_needed)
                                .map(|_| "{}")
                                .collect::<Vec<&str>>()
                                .join(" "),
                            deconstruct.replace('(', "").replace(')', "")
                        ));
                    }
                }
            }
        }

        // do the From stuff
        let from_kind = if k == "()" {
            "u8".to_owned()
        } else {
            format!("(u8, {})", k)
        };
        let tpl_value = if k == "()" {
            "".to_owned()
        } else if k.starts_with('(') && k.contains(',') {
            let count = tuple_count(k);
            format!(
                "({})",
                (0..count)
                    .map(|i| format!("value.1.{}", i))
                    .collect::<Vec<String>>()
                    .join(",")
            )
        } else {
            "(value.1)".to_owned()
        };
        output.push(format!(
            "impl From<{from_kind}> for Operation {{
    fn from(value: {from_kind}) -> Self {{
            match value{} {{{}, _ => panic!()}}
    }}
}}
",
            if tpl_value != "" { ".0" } else { "" },
            v.operations
                .iter()
                .map(|(i, op)| format!("{} => Operation::{}{tpl_value}", i, op.iden.to_string()))
                .collect::<Vec<String>>()
                .join(",")
        ))
    }

    output.push(format!(
        "impl Display for Operation {{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {{
        match self {{
            {}
        }}
    }}
}}
",
        formatting_stuff.join(",")
    ));

    TokenStream::from_str(&output.join("\n")).expect("could not make the Froms and Displays")
}

pub fn generate_name_things(slice: &[OperationDefinition]) -> TokenStream {
    let name_indices: Vec<(usize, String)> = slice
        .iter()
        .enumerate()
        .map(|(i, x)| (i, x.snake_case.clone()))
        .collect();

    let get_opcode = format!(
        "pub fn get_opcode(op: &str) -> usize {{match op {{{0}, _ => todo!(\"{{}}\", op)}}}}",
        name_indices
            .iter()
            .map(|(i, n)| format!("\"{n}\" => {i}"))
            .collect::<Vec<String>>()
            .join(",")
    );
    let exists = format!(
        "pub fn exists(op: &str) -> bool {{ {} }}",
        name_indices
            .iter()
            .map(|(_, n)| format!("op == \"{n}\""))
            .collect::<Vec<String>>()
            .join("||")
    );

    return TokenStream::from_str(&format!("impl Operation {{{}\n{}}}", get_opcode, exists))
        .expect("couldn't do the name things");
}

pub fn generate_if_op(iden: &Ident, thing: &Vec<TokenTree>) -> String {
    let first = thing.first();
    match first {
        Some(TokenTree::Group(g)) => {
            assert_eq!(
                g.delimiter(),
                Delimiter::Brace,
                "only blocks have been implemented"
            );
            format!(
                "\"{}\" => {},",
                snake_case(&iden.to_string()),
                g.to_string()
            )
        }
        Some(_) => {
            format!(
                "\"{}\" => Operation::{}({}),",
                snake_case(&iden.to_string()),
                iden.to_string(),
                TokenStream::from_iter(thing.iter().map(|x| x.clone())).to_string()
            )
        }
        None => {
            format!(
                "\"{}\" => Operation::{},",
                snake_case(&iden.to_string()),
                iden.to_string(),
            )
        }
    }
}
