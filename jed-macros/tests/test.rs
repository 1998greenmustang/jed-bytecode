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
    fn display_test() {
        #[derive(Debug, SnakeCaseDisplay)]
        enum Test {
            #[jed(type: &'static [u8], func: bytes_display )]
            Variant,
            TypedVariant(&'static [u8]),
            TupledVariant(&'static [u8], u8),
            #[rustfmt::skip]
            TrailingComma(&'static [u8],),
        }
    }

    #[test]
    fn exists_test() {
        #[derive(Debug, SnakeCaseExists)]
        enum Test {
            #[jed(type: &'static [u8], func: bytes_display )]
            Variant,
            TypedVariant(&'static [u8]),
            TupledVariant(&'static [u8], u8),
            #[rustfmt::skip]
            TrailingComma(&'static [u8],),
        }
    }

    #[test]
    fn index_to_name_test() {
        #[derive(Debug, IndexToSnakeCase)]
        enum Test {
            Variant,
            TypedVariant(&'static [u8]),
            TupledVariant(&'static [u8], u8),
            #[rustfmt::skip]
            TrailingComma(&'static [u8],),
        }
    }

    #[test]
    fn index_froms_test() {
        #[derive(Debug, IndexFroms)]
        enum Test {
            Variant,
            TypedVariant(&'static [u8]),
            TupledVariant(&'static [u8], u8),
            #[rustfmt::skip]
            TrailingComma(&'static [u8],),
        }
    }
}
