use std::collections::{HashMap, HashSet};
use std::fmt;

use node::*;

pub struct TransTable {
    start: usize,
    end: HashSet<usize>,
    states: HashSet<usize>,
    trans: HashMap<usize, Vec<Edge>>,
}

fn cut_epsilon(table: &mut TransTable) -> TransTable {
    unimplemented!()
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

    table
        .trans
        .entry(start_id)
        .or_insert(vec![])
        .append(&mut start.edges().clone());
    table
        .trans
        .entry(end_id)
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

    fn epsilon_move(&self, state: usize) -> HashSet<usize> {
        let mut r: HashSet<usize> = HashSet::new();

        self.epsilon_move_internal(state, &mut r);

        r.iter()
         .filter(|&x| *x != state && self.has_nontrivial_edge(*x))
         .map(|x| *x)
         .collect()
    }

    fn epsilon_move_internal(&self, state: usize, visited: &mut HashSet<usize>) {
        if visited.contains(&state) { return; }
        visited.insert(state);

        for e in self
            .trans
            .get(&state)
            .unwrap()
            .iter()
            .filter(|x| x.matches().is_none())
        {
            self.epsilon_move_internal(e.next_node(), visited);
        }
    }

    fn has_nontrivial_edge(&self, state: usize) -> bool {
        self.trans
            .get(&state)
            .unwrap()
            .iter()
            .any(|x| x.matches().is_some())
    }
}

impl fmt::Display for TransTable {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let mut states: Vec<usize> = self.states.iter().map(|x| *x).collect();
        states.sort();
        let pos = move |x: &usize| states.iter().position(|y| y == x).unwrap();

        writeln!(f, "TransTable(start: 0)")?;

        // dump states
        let mut states = self
            .states
            .iter()
            .map(|x| *x)
            .collect::<Vec<usize>>();
        states.sort();

        for state in states.iter() {
            if self.end.contains(state) {
                writeln!(f, "\tState {}*", pos(state))?;
            } else {
                writeln!(f, "\tState {}", pos(state))?;
            }

            if let Some(edges) = self.trans.get(state) {
                for edge in edges {
                    writeln!(
                        f,
                        "\t\tmatch {} to {}",
                        edge.matches()
                            .clone()
                            .map(|x| x.to_string())
                            .unwrap_or("\u{03b5}".to_string()),
                        pos(&edge.next_node())
                    )?;
                }
            }
        }

        writeln!(f)
    }
}

#[cfg(test)]
mod test {
    use regex_gen::RegexItem;
    use transtable::*;

    macro_rules! assert_move {
        ($table: expr, $state: expr, $expect: expr) => {{
            let mut states: Vec<usize> = $table.states.iter().map(|x| *x).collect();
            states.sort();

            let mut l: Vec<usize> = $expect;
            let mut r: Vec<usize> = $table
                .epsilon_move(states[$state])
                .iter()
                .map(|x| states.iter().position(|y| y == x).unwrap())
                .collect();

            l.sort();
            r.sort();

            assert_eq!(r, l);
        }};
    }

    #[test]
    fn test_epsilon_move() {
        let r: RegexItem = r#"(a|b)+c"#.into();
        let t = TransTable::from_nfa(&r.nfa_graph());
        assert_eq!(t.states.len(), 8);
        assert_move!(t, 0, vec![2, 4]);
        assert_move!(t, 3, vec![6, 2, 4]);
        assert_move!(t, 5, vec![6, 2, 4]);
        assert_move!(t, 1, vec![6, 2, 4]);
        assert_move!(t, 6, vec![]);
        assert_move!(t, 7, vec![]);

        let r: RegexItem = r#"[-c]*"#.into();
        let t = TransTable::from_nfa(&r.nfa_graph());
        assert_eq!(t.states.len(), 6);
        assert_move!(t, 0, vec![2, 4]);
        assert_move!(t, 1, vec![2, 4]);
        assert_move!(t, 2, vec![]);
        assert_move!(t, 4, vec![]);
        assert_move!(t, 3, vec![2, 4]);
        assert_move!(t, 5, vec![2, 4]);

        let r: RegexItem = r#"([ab]+|c*)?"#.into();
        let t = TransTable::from_nfa(&r.nfa_graph());
        assert_eq!(t.states.len(), 10);
        assert_move!(t, 0, vec![4, 6, 8]);
        assert_move!(t, 2, vec![4, 6]);
        assert_move!(t, 8, vec![]);
        assert_move!(t, 5, vec![4, 6]);
    }
}
