use crate::object::Object;

#[repr(u8)]
#[derive(Debug, Clone, Copy)]
pub enum BinOpKind {
    Add,
    Sub,
    Mul,
    Div,
    Mod,

    Eq,
    LessEq,
    GreatEq,
    Lesser,
    Greater,
}

impl BinOpKind {
    pub fn from_object(arg: Object) -> BinOpKind {
        match arg {
            val if val.1 == b"+" => BinOpKind::Add,
            val if val.1 == b"-" => BinOpKind::Sub,
            val if val.1 == b"*" => BinOpKind::Mul,
            val if val.1 == b"/" => BinOpKind::Div,
            val if val.1 == b"==" => BinOpKind::Eq,
            val if val.1 == b"<=" => BinOpKind::LessEq,
            val if val.1 == b">=" => BinOpKind::GreatEq,
            val if val.1 == b"<" => BinOpKind::Lesser,
            val if val.1 == b">" => BinOpKind::Greater,
            val if val.1 == b"%" => BinOpKind::Mod,
            _ => panic!("Not implemented: {:?}", arg),
        }
    }
}
