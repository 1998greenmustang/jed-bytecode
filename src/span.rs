use std::fmt::Display;

use crate::operation::Operation;

#[derive(Clone, Debug)]
pub struct Span {
    pub program_count: usize,
    pub current_op: Operation,
    pub prev_op: Operation,
    pub next_op: Operation,
}

impl Span {
    pub fn empty() -> Self {
        Self {
            program_count: 0,
            current_op: Operation::Empty,
            prev_op: Operation::Empty,
            next_op: Operation::Empty,
        }
    }
}

impl Display for Span {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "\t{}  {}\n",
            self.program_count.checked_sub(1).unwrap_or_default(),
            self.prev_op,
        )?;
        write!(f, ">>>\t{}  {}\n", self.program_count, self.current_op)?;
        write!(f, "\t{}  {}", self.program_count + 1, self.next_op)
    }
}
