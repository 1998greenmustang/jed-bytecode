#![allow(dead_code)]
pub mod saveable;

use crate::object::{Object, ObjectKind};
use crate::operation::*;
use crate::program::Program;
use std::convert::TryInto;
use std::fs::File;
use std::io::{self, Read, Write};

pub fn read_file(file: File) -> Program {}
