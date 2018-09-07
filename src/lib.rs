
use std::io;
use std::io::Write;

#[macro_use]

extern crate matches;

trait CodeGenerator {
    fn generate<W: Write>(&self, w: &mut W) -> io::Result<()>;
}

mod terminal;
pub mod regex_gen;
mod node;
mod nfa;
pub mod transtable;

