use std::{collections::HashMap, str::FromStr};

use proc_macro::TokenStream;

use crate::ast::{Ast, AstEntry};

pub fn enum_display<T: Fn(&String) -> String>(ast: Ast, id_fn: T) -> TokenStream {
    let mut attrs: HashMap<String, String> = HashMap::new();
    let mut write_stmts: Vec<String> = Vec::new();
    if let Some(AstEntry::Enum(name, ast)) = ast.first() {
        for node in ast {
            match node {
                AstEntry::Variant(vname, tpl) => {
                    let mut fmt_string = format!("{}", id_fn(vname));
                    let deconstruct = if tpl.len() > 0 {
                        let args: Vec<String> = (0..tpl.len())
                            .map(|i| "a".to_owned() + &i.to_string())
                            .collect();
                        "(".to_owned() + &args.join(",") + ")"
                    } else {
                        "".to_owned()
                    };
                    let mut args = Vec::new();
                    for (i, k) in tpl.iter().enumerate() {
                        fmt_string += " {}";
                        let arg = "a".to_owned() + &i.to_string();
                        args.push(if let Some(f) = attrs.get(&k.to_string()) {
                            format!("{f}({arg})")
                        } else {
                            arg
                        });
                    }
                    write_stmts.push(format!(
                        "\t    {name}::{vname}{deconstruct} => write!(f, \"{fmt_string}\", {})",
                        args.join(",")
                    ));
                }
                AstEntry::Attribute(_, attr_map) => {
                    if let Some(kind) = attr_map.get("type") {
                        if let Some(func) = attr_map.get("func") {
                            attrs.insert(kind.to_string(), func.to_string());
                        }
                    }
                }
                _ => todo!("{node:?}"),
            }
        }
        let output = format!(
            "impl Display for {name} {{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {{
        match self {{
{},
            _ => todo!(\"{{self:?}}\")
        }}
    }}
}}
",
            write_stmts.join(",\n")
        );
        return TokenStream::from_str(&output).expect("im so valid");
    };
    unreachable!()
}
