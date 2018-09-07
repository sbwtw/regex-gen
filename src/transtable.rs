
use std::collections::{HashMap, HashSet};
use std::fmt;

use node::*;

pub struct TransTable {
    start: usize,
    end: HashSet<usize>,
    states: HashSet<usize>,
    trans: HashMap<usize, Vec<Edge>>,
}

fn append_states(table: &mut TransTable, nfa: &NFAGraph) {
    table.states.insert(nfa.start_id());
    table.states.insert(nfa.end_id());

    for n in nfa.sub_graphs() {
        append_states(table, n);
    }
}

fn append_trans(table: &mut TransTable, nfa: &NFAGraph) {
    let (start, end) = nfa.nodes();
    let start_id = nfa.start_id();
    let end_id = nfa.end_id();

    table.trans.insert(start_id, start.edges().clone());
    table.trans.insert(end_id, end.edges().clone());

    for n in nfa.sub_graphs() {
        append_states(table, n);
    }
}

impl TransTable {
    pub fn from_nfa(nfa: &NFAGraph) -> TransTable {
        let mut r = TransTable {
            start: nfa.start_id(),
            end: HashSet::new(),
            states: HashSet::new(),
            trans: HashMap::new(),
        };

        r.end.insert(nfa.end_id());

        r
    }
}

impl fmt::Display for TransTable {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        writeln!(f, "TransTable(start: {})", self.start)?;

        // dump states
        write!(f, "States: ")?;
        for state in self.states.iter() {
            if self.end.contains(state) {
                write!(f, "{}*\t", state)?;
            } else {
                write!(f, "{}\t", state)?;
            }
        }
        writeln!(f)?;

        // dump edges
        writeln!(f, "Edges:")?;

        writeln!(f)
    }
}

