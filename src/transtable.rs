
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

    table.trans.entry(start_id)
        .or_insert(vec![])
        .append(&mut start.edges().clone());
    table.trans.entry(end_id)
        .or_insert(vec![])
        .append(&mut end.edges().clone());

    for n in nfa.sub_graphs() {
        append_trans(table, n);
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
        append_states(&mut r, nfa);
        append_trans(&mut r, nfa);

        r
    }

    pub fn state_count(&self) -> usize {
        self.states.len()
    }

    pub fn edge_count(&self) -> usize {
        self.trans.values().map(|x| x.len()).sum()
    }
}

impl fmt::Display for TransTable {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        writeln!(f, "TransTable(start: {})", self.start)?;

        // dump states
        for state in self.states.iter() {
            if self.end.contains(state) {
                writeln!(f, "\tState {}*", state)?;
            } else {
                writeln!(f, "\tState {}", state)?;
            }

            if let Some(edges) = self.trans.get(state) {
                for edge in edges {
                    writeln!(f, "\t\tmatch '{}' to {}",
                             edge.matches().map(|x| x as char).unwrap_or('ε'),
                             edge.next_node())?;
                }
            }
        }

        writeln!(f)
    }
}

