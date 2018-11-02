
extern crate itertools;

macro_rules! set {
    ($($x: expr),*) => {
        {
            use std::collections::BTreeSet;

            let mut s = BTreeSet::new();
            $(
                s.insert($x);
            )*

            s
        }
    };
}

pub mod regex_gen;
mod node;
pub mod transtable;
pub mod dot_graph;
pub mod execute_engine;

