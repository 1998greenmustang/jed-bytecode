#[cfg(test)]
mod tests {
    extern crate jed_macros;
    use jed_macros::*;
    use std::fmt::Display;

    fn option_display(a: &Option<usize>) -> String {
        format!("{:?}", a)
    }
    fn bytes_display(a: &'static [u8]) -> String {
        format!("{:?}", a)
    }
    fn func_display(a: &'static [u8], b: &usize) -> String {
        format!("{:?} {}", a, b)
    }

    #[test]
    fn register_test() {
        match_ops!( {Whatever, func_display}, {[
                    Empty,
                    Pop,
                    Done,
                    Exit,
                    DoFor,
                    PushTemp,
                    StoreTemp,
                    ListPush,
                    PushRange,
                    GetPtr,
                    ReadPtr,
                    SetPtr,
                    GetIter,
                    IterNext,
                    IterPrev,
                    IterSkip,
                    IterCurrent,
                    Iterate,
                    DoIf,
                    Debug
                ]},);
    }

    #[test]
    fn teasduijh() {
        create_operations!(
                (): Empty,
                    Pop,
                    Done,
                    Exit,
                    DoFor,
                    PushTemp,
                    StoreTemp,
                    ListPush,
                    PushRange,
                    GetPtr,
                    ReadPtr,
                    SetPtr,
                    GetIter,
                    IterNext,
                    IterPrev,
                    IterSkip,
                    IterCurrent,
                    Iterate,
                    DoIf,
                    Debug;
                (u16): BinOp;
                (u8): CallBuiltIn;
                (&'static [u8], usize) func_display: Func;
                (&'static [u8]) bytes_display: Call,
                               PushLit,
                               PushName,
                               ReturnIf,
                               StoreConst,
                               StoreName,
                               DoForIn,
                               ReturnIfConst,
                               Import;
                (Option<usize>) option_display: CreateList,
                               ListGet,
                               ListSet,
                               ListAlloc;
        );
        let _ = Operation::Empty;
    }
}
