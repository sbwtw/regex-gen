use std::collections::{HashMap, HashSet};
use std::fmt;

use node::*;

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

    table.append_edges(start_id, &mut start.edges().clone());
    table.append_edges(end_id, &mut end.edges().clone());

    for n in nfa.sub_graphs() {
        append_trans(table, n);
    }
}

pub struct TransTable {
    start: usize,
    end: HashSet<usize>,
    states: HashSet<usize>,
    trans: HashMap<usize, Vec<Edge>>,
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

    pub fn cut_epsilon(&mut self) {
        // mark epsilon move as end state
        {
            // collect state epsilon move
            let epsilon_move: Vec<(usize, HashSet<usize>)> = self.states
                .iter()
                .map(|x| (*x, self.epsilon_move(*x)))
                .collect();

            assert!(self.end.len() == 1);
            let end = self.end.iter().next().unwrap().clone();

            for (state, dests) in epsilon_move {
                if dests.contains(&end) {
                    self.end.insert(state);
                }
            }
        }

        // generate edges
        for (state, mut edges) in self
            .states
            .iter()
            .filter(|&x| self.has_epsilon_edge(*x))
            .map(|x| (*x, self.posssible_nontrivial_edges(*x)))
            .collect::<Vec<(usize, Vec<Edge>)>>() {
            self.append_edges(state, &mut edges);
        }

        // collect all useful states
        let mut useful_states = vec![self.start];
        let mut visit = vec![self.start];
        while let Some(state) = visit.pop() {
            for e in self.trans.get(&state).unwrap() {
                let n = e.next_node();
                if !useful_states.contains(&n) && e.matches().is_some() {
                    useful_states.push(n);
                    visit.push(n);
                }
            }
        }

        // remove no-used states
        self.states.retain(|x| useful_states.contains(x));
        self.trans.retain(|x, _| useful_states.contains(x));

        // remove epsilon edges
        for (_, mut edges) in self.trans.iter_mut() {
            edges.retain(|e| e.matches().is_some());
        }
    }

    fn append_edges(&mut self, state: usize, edges: &mut Vec<Edge>) {
        self.trans.entry(state).or_insert(vec![]).append(edges);
    }

    fn posssible_nontrivial_edges(&self, state: usize) -> Vec<Edge> {
        self.epsilon_move(state)
            .iter()
            .map(|x| self.trans.get(&x).unwrap())
            .flatten()
            .map(|x| x.clone())
            .collect()
    }

    fn epsilon_move(&self, state: usize) -> HashSet<usize> {
        let mut r: HashSet<usize> = HashSet::new();

        self.epsilon_move_internal(state, &mut r);

        r.iter()
            .filter(|&x| *x != state && (self.end.contains(x) || self.has_nontrivial_edge(*x)))
            .map(|x| *x)
            .collect()
    }

    fn epsilon_move_internal(&self, state: usize, visited: &mut HashSet<usize>) {
        if visited.contains(&state) {
            return;
        }
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

    fn has_epsilon_edge(&self, state: usize) -> bool {
        self.trans
            .get(&state)
            .unwrap()
            .iter()
            .any(|x| x.matches().is_none())
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
        let mut states = self.states.iter().map(|x| *x).collect::<Vec<usize>>();
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
    fn test_cut_epsilon() {
        let r: RegexItem = r#"(a|b)+c"#.into();
        let mut t = TransTable::from_nfa(&r.nfa_graph());
        t.cut_epsilon();
        assert_eq!(t.state_count(), 4);
        assert_eq!(t.edge_count(), 8);

        let r: RegexItem = r#"([ab]+|c*)?"#.into();
        let mut t = TransTable::from_nfa(&r.nfa_graph());
        t.cut_epsilon();
        assert_eq!(t.state_count(), 4);
        assert_eq!(t.edge_count(), 8);
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
