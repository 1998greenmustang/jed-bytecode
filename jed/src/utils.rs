use crate::error::ProgramErrorKind;

pub fn bytes_to_string(bytes: &[u8]) -> String {
    match String::from_utf8(bytes.to_vec()) {
        Ok(s) => s,
        Err(_) => unreachable!(),
    }
}

pub fn display_option_usize(a: &Option<usize>) -> String {
    match a {
        Some(n) => n.to_string(),
        None => "".to_owned(),
    }
}

pub fn unwrap_or_error<T>(
    option: Option<T>,
    kind: ProgramErrorKind,
) -> Result<T, ProgramErrorKind> {
    match option {
        Some(v) => Ok(v),
        None => Err(kind),
    }
}

pub fn string_to_t<T>(str: String) -> Result<T, ProgramErrorKind>
where
    T: std::str::FromStr,
{
    match str.parse::<T>() {
        Ok(v) => Ok(v),
        Err(_) => Err(ProgramErrorKind::ParsingError(str)),
    }
}

pub fn string_is_float_like(str: String) -> bool {
    let mut has_dot = false;
    for c in str.chars() {
        if c == '.' {
            if !has_dot {
                has_dot = true;
            } else {
                return false;
            }
        }
        if c.is_numeric() {
            continue;
        } else {
            break;
        }
    }
    return has_dot;
}

pub fn isize_to_usize(i: isize) -> usize {
    isize::cast_unsigned(i)
}
