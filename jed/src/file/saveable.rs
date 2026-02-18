use std::{
    fs::File,
    io::{self, Read, Write},
};

use crate::{file::OperationSaveable, program::Program};

use super::ProgramFile;
unsafe fn any_as_u8_slice<T: Sized>(p: &T) -> &[u8] {
    core::slice::from_raw_parts((p as *const T) as *const u8, core::mem::size_of::<T>())
}

pub trait Saveable
where
    Self: Sized,
{
    fn save(&self, filepath: &str) -> io::Result<()>;
    fn load(filepath: &str) -> io::Result<Self>;
}

impl Saveable for ProgramFile {
    fn save(&self, filepath: &str) -> io::Result<()> {
        let mut file = File::create(filepath)?;

        let bytes: &[u8] = unsafe { any_as_u8_slice(&self.0) };
        file.write_all(bytes)?;
        Ok(())
    }

    fn load(filepath: &str) -> io::Result<Self> {
        let mut file = File::open(filepath)?;

        let bytes: &mut [u8; std::mem::size_of::<Vec<OperationSaveable>>()] =
            &mut [0 as u8; std::mem::size_of::<Vec<OperationSaveable>>()];
        File::read(&mut file, bytes)?;
        let data: *const [u8; std::mem::size_of::<Vec<OperationSaveable>>()] =
            bytes as *const [u8; std::mem::size_of::<Vec<OperationSaveable>>()];

        let this = Self(unsafe { std::mem::transmute(*data) });

        assert!(this.0.len() > 0, "no instructions!");
        Ok(this)
    }
}
impl Saveable for Program {
    fn save(&self, filepath: &str) -> io::Result<()> {
        ProgramFile::from(self).save(filepath)
    }

    fn load(filepath: &str) -> io::Result<Self> {
        let file = ProgramFile::load(filepath)?;
        Ok(Self::from(file))
    }
}
