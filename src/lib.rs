
use std::io;
use std::io::Write;

trait CodeGenerator {
    fn generate<W: Write>(&self, w: &mut W) -> io::Result<()>;
}

pub mod regex_gen;
mod node;
pub mod transtable;
pub mod dot_graph;

