use crate::error::ProgramErrorKind;

pub fn bytes_to_string(bytes: &[u8]) -> String {
    match String::from_utf8(bytes.to_vec()) {
        Ok(s) => s,
        Err(_) => unreachable!(),
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

pub fn unwrap_as_string_or<T>(option: Option<T>, or: &str) -> String
where
    T: ToString,
{
    match option {
        Some(v) => v.to_string(),
        None => or.to_owned(),
    }
}

pub fn isize_to_usize(i: isize) -> usize {
    unsafe { std::mem::transmute(i) }
}

pub fn bounded(min: usize, v: usize, max: usize) -> bool {
    return v >= min && v <= max;
}
