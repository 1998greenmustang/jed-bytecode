use std::{collections::HashMap, iter::Peekable};

use proc_macro::{Delimiter, Ident, Literal, TokenStream, TokenTree, token_stream};

use crate::{is_group, is_ident, is_punct};

pub(crate) type Ast = Vec<AstEntry>;

#[derive(Clone, Debug)]
pub enum AstEntry {
    Enum(String, Vec<AstEntry>),
    Variant(String, Vec<AstEntry>),
    Attribute(String, HashMap<String, AstEntry>),
    List(Vec<AstEntry>),
    Expr(AstExpr),
    Ident(Ident),
    Literal(Literal),
    Kind(AstKind),
}

impl ToString for AstEntry {
    fn to_string(&self) -> String {
        match self {
            AstEntry::Expr(e) => e.to_string(),
            AstEntry::Kind(k) => k.to_string(),
            _ => todo!("{self:?}"),
        }
    }
}

pub(crate) type AstExpr = String;
pub(crate) type AstKind = String;

type AstResult<T> = Result<T, AstError>;

#[derive(Debug)]
pub enum AstError {
    TodoError(String),
}

fn parse_expr(iter: &mut Peekable<token_stream::IntoIter>) -> AstResult<AstEntry> {
    let mut collected = String::new();
    // lazy method: collect until , or end of stream
    while let Some(tt) = iter.peek() {
        match tt {
            TokenTree::Punct(p) if p.as_char() == ',' => break,
            a => {
                collected += &a.to_string();
                iter.next()
            }
        };
    }
    Ok(AstEntry::Expr(collected))
}
fn parse_type(iter: &mut Peekable<token_stream::IntoIter>) -> AstResult<AstEntry> {
    let mut collected = String::new();
    // lazy method: collect until , or end of stream
    while let Some(tt) = iter.peek() {
        match tt {
            TokenTree::Punct(p) if p.as_char() == ',' => break,
            a => {
                collected += &a.to_string();
                iter.next()
            }
        };
    }
    Ok(AstEntry::Kind(collected))
}

fn parse_variant_types(
    mut iter: &mut Peekable<token_stream::IntoIter>,
) -> AstResult<Vec<AstEntry>> {
    let mut types: Vec<AstEntry> = vec![parse_type(&mut iter)?];
    while let Some(_comma) = iter.next_if(|x| is_punct(x, ',')) {
        if let Some(_) = iter.peek() {
            types.push(parse_type(&mut iter)?)
        }
    }
    Ok(types)
}

fn parse_attributes(attr: (TokenTree, TokenTree)) -> AstResult<AstEntry> {
    // #[jed(thing: )]
    assert!(is_ident(&attr.0));
    assert!(is_group(&attr.1, Delimiter::Parenthesis));

    if let TokenTree::Group(g) = attr.1 {
        let mut iter = g.stream().into_iter().peekable();
        let mut map: HashMap<String, AstEntry> = HashMap::new();
        while let Some(next) = iter.next() {
            match next {
                TokenTree::Ident(id) => {
                    if let Some(_colon) = iter.next_if(|x| is_punct(x, ':')) {
                        if id.to_string() == "type" {
                            map.insert(id.to_string(), parse_type(&mut iter)?);
                        } else {
                            let mut list_of_exprs: Vec<AstEntry> = vec![parse_expr(&mut iter)?];
                            while let Some(_comma) = iter.next_if(|x| is_punct(x, ',')) {
                                list_of_exprs.push(parse_expr(&mut iter)?);
                            }
                            if list_of_exprs.len() > 1 {
                                map.insert(id.to_string(), AstEntry::List(list_of_exprs));
                            } else if list_of_exprs.len() == 1 {
                                map.insert(id.to_string(), list_of_exprs.get(0).unwrap().clone());
                            }
                        }
                    } else {
                        return Err(AstError::TodoError("missing colon".to_owned()));
                    }
                }
                TokenTree::Punct(p) if p.as_char() == ',' => continue,
                _ => return Err(AstError::TodoError(format!("no name bro {next}"))),
            }
        }

        Ok(AstEntry::Attribute(attr.0.to_string(), map))
    } else {
        Err(AstError::TodoError("whats evven going on".to_owned()))
    }
}

fn parse_enum(iter: &mut Peekable<token_stream::IntoIter>) -> AstResult<AstEntry> {
    if let Some(TokenTree::Ident(name)) = iter.next() {
        if let Some(TokenTree::Group(g)) = iter.next_if(|x| is_group(x, Delimiter::Brace)) {
            let mut g_iter = g.stream().into_iter().peekable();
            let mut current_tree: Vec<AstEntry> = Vec::new();
            while let Some(tt) = g_iter.next() {
                match tt {
                    TokenTree::Ident(ident) => {
                        if let Some(TokenTree::Group(grp)) =
                            g_iter.next_if(|x| is_group(x, Delimiter::Parenthesis))
                        {
                            current_tree.push(AstEntry::Variant(
                                ident.to_string(),
                                parse_variant_types(&mut grp.stream().into_iter().peekable())?,
                            ));
                        } else {
                            current_tree.push(AstEntry::Variant(ident.to_string(), Vec::new()));
                        }
                    }
                    TokenTree::Punct(punct) if punct.as_char() == '#' => {
                        if let Some(TokenTree::Group(der)) =
                            g_iter.next_if(|x| is_group(x, Delimiter::Bracket))
                        {
                            let mut d_iter = der.stream().into_iter().peekable();
                            if let Some(id) = d_iter.next_if(is_ident) {
                                if id.to_string() == "jed" {
                                    if let Some(g) =
                                        d_iter.next_if(|x| is_group(x, Delimiter::Parenthesis))
                                    {
                                        current_tree.push(parse_attributes((id, g))?);
                                    }
                                }
                            }
                        }
                    }
                    TokenTree::Punct(punct) if punct.as_char() == ',' => continue,
                    TokenTree::Literal(literal) => todo!("literal {}", literal),
                    _ => todo!("idk what this is {}", tt),
                }
            }
            return Ok(AstEntry::Enum(name.to_string(), current_tree));
        }
    }
    Err(AstError::TodoError("couldnt parse enum".to_owned()))
}

pub fn parse(stream: TokenStream) -> AstResult<Ast> {
    let mut iter = stream.into_iter().peekable();
    loop {
        match iter.next() {
            Some(TokenTree::Ident(kw)) => {
                let keyword: &str = &kw.to_string();
                match keyword {
                    "enum" => return Ok(vec![parse_enum(&mut iter)?]),
                    "pub" => continue,
                    _ => todo!("parsing {}", kw),
                }
            }
            Some(TokenTree::Punct(p)) if p.as_char() == '#' => continue,
            Some(TokenTree::Group(g)) if g.delimiter() == Delimiter::Bracket => continue,
            _ => {
                return Err(AstError::TodoError(format!(
                    "{:?} is straight up not a block",
                    iter.next()
                )));
            }
        }
    }
}
