extern crate proc_macro;
mod generators;
mod utils;
use std::{collections::HashMap, str::FromStr};

use proc_macro::*;

use generators::*;
use utils::*;

type OperationEntry = (usize, OperationDefinition);

#[derive(Clone)]
struct KindStuff {
    kind: TokenTree,
    display_func: Option<TokenTree>,
    operations: Vec<OperationEntry>,
}

#[derive(Clone, Debug)]
struct OperationDefinition {
    iden: Ident,
    kind: TokenTree,
    snake_case: String,
}

impl OperationDefinition {
    fn new(iden: Ident, kind: TokenTree) -> Self {
        Self {
            iden: iden.clone(),
            kind,
            snake_case: snake_case(&iden.to_string()),
        }
    }
}

#[proc_macro]
pub fn create_operations(stream: TokenStream) -> TokenStream {
    let mut ops: Vec<OperationDefinition> = vec![];
    let mut kinds: HashMap<String, KindStuff> = HashMap::new();
    let mut iter = stream.into_iter().peekable();
    loop {
        // (type) opt_display_function: EnumMember, ...;
        if let Some(kind) = iter.next_if(|x| is_kind(x)) {
            let mut kstuff = KindStuff {
                kind: kind.clone(),
                display_func: iter.next_if(|x| is_ident(x)),
                operations: vec![],
            };

            if let Some(_colon) = iter.next_if(|x| is_punct(x, ':')) {
                let idens: &[Ident] = &collect_idents(&mut iter);
                if let Some(_semi) = iter.next_if(|x| is_punct(x, ';')) {
                    for id in idens {
                        let def = OperationDefinition::new(id.clone(), kind.clone());
                        kstuff.operations.push((ops.len(), def.clone()));
                        ops.push(def)
                    }
                } else {
                    break;
                }
            }
            kinds.insert(kind.to_string(), kstuff);
        } else {
            break;
        }
    }

    let mut enum_def = generate_enum(&ops);
    let from_def = generate_kind_things(kinds);
    let name_def = generate_name_things(&ops);
    enum_def.extend(from_def);
    enum_def.extend(name_def);
    return enum_def;
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
        let second: Vec<TokenTree> = {
            let mut v = Vec::new();
            while let Some(tt) = g_iter.next() {
                v.push(tt);
            }
            v
        };

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
    println!("{:?}", outputs);
    return TokenStream::from_str(&outputs.join("\n")).expect("comon");
}
