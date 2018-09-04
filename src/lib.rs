
use std::io;
use std::io::Write;

#[macro_use]
extern crate matches;

trait CodeGenerator {
    fn generate<W: Write>(&self, w: &mut W) -> io::Result<()>;
}

pub mod regex_gen;

