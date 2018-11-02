use std::collections::{HashMap, HashSet};
use std::fmt;

use itertools::*;
use node::*;

fn append_states(table: &mut TransTable, nfa: &NFAGraph) {
    table.states.insert(set![nfa.start_id()]);
    table.states.insert(set![nfa.end_id()]);

    for n in nfa.sub_graphs() {
        append_states(table, n);
    }
}

fn append_trans(table: &mut TransTable, nfa: &NFAGraph) {
    let (start, end) = nfa.nodes();
    let start_id = nfa.start_id();
    let end_id = nfa.end_id();

    table.append_edges(&set![start_id], &mut start.edges().clone());
    table.append_edges(&set![end_id], &mut end.edges().clone());

    for n in nfa.sub_graphs() {
        append_trans(table, n);
    }
}

pub struct TransTable {
    start: States,
    end: HashSet<States>,
    states: HashSet<States>,
    trans: HashMap<States, Vec<Edge>>,
}

impl TransTable {
    pub fn from_nfa(nfa: &NFAGraph) -> TransTable {
        let mut r = TransTable {
            start: set![nfa.start_id()],
            end: HashSet::new(),
            states: HashSet::new(),
            trans: HashMap::new(),
        };

        r.end.insert(set![nfa.end_id()]);
        append_states(&mut r, nfa);
        append_trans(&mut r, nfa);

        r
    }

    pub fn start_id(&self) -> &States {
        &self.start
    }

    pub fn state_count(&self) -> usize {
        self.states.len()
    }

    pub fn edge_count(&self) -> usize {
        self.trans.values().map(|x| x.len()).sum()
    }

    pub fn end_set(&self) -> &HashSet<States> {
        &self.end
    }

    pub fn trans_map(&self) -> &HashMap<States, Vec<Edge>> {
        &self.trans
    }

    pub fn as_dfa(&mut self) {
        // mark epsilon move as end state
        {
            // collect state epsilon move
            let epsilon_move: Vec<(States, HashSet<States>)> = self
                .states
                .iter()
                .map(|x| (x.clone(), self.epsilon_move(x)))
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
            .filter(|&x| self.has_epsilon_edge(&*x))
            .map(|x| (x.clone(), self.posssible_nontrivial_edges(&*x)))
            .collect::<Vec<(States, Vec<Edge>)>>()
        {
            self.append_edges(&state, &mut edges);
        }

        // collect all useful states
        let mut useful_states = vec![self.start.clone()];
        let mut visit = vec![self.start.clone()];
        while let Some(state) = visit.pop() {
            for e in self.trans.get(&state).unwrap() {
                let n = e.next_node();
                if !useful_states.contains(&n) && e.matches().is_some() {
                    useful_states.push(n.clone());
                    visit.push(n.clone());
                }
            }
        }

        // remove no-used states
        self.states.retain(|x| useful_states.contains(x));
        self.end.retain(|x| useful_states.contains(x));
        self.trans.retain(|x, _| useful_states.contains(x));

        // remove epsilon edges
        for (_, mut edges) in self.trans.iter_mut() {
            edges.retain(|e| e.matches().is_some());
        }

        // test edges intersect
        let intersected = self.trans.values().any(|edges| {
            edges
                .iter()
                .combinations(2)
                .any(|pair| pair[0].intersect(pair[1]))
        });
        if !intersected { return; }

    }

    pub fn reset_state_mark(&mut self) {
        let mut states: Vec<States> = self.states.iter().map(|x| x.clone()).collect();
        states.sort();

        let mut m = HashMap::new();
        for (index, state) in states.iter().enumerate() {
            m.insert(state, set![index]);
        }

        let pos = move |x: &States| m.get(&*x).unwrap().clone();

        self.start = pos(&self.start);
        self.end = self.end.iter().map(|x| pos(&*x)).collect();
        self.states = self.states.iter().map(|x| pos(&*x)).collect();
        self.trans = self
            .trans
            .iter()
            .map(|(state, edges)| {
                (
                    pos(&*state),
                    edges
                        .iter()
                        .map(|x| Edge::new(pos(x.next_node()), x.matches().clone()))
                        .collect(),
                )
            })
            .collect();
    }

    fn append_edges(&mut self, state: &States, edges: &mut Vec<Edge>) {
        self.trans.entry(state.clone()).or_insert(vec![]).append(edges);
    }

    fn posssible_nontrivial_edges(&self, state: &States) -> Vec<Edge> {
        Iterator::flatten(
            self.epsilon_move(state)
                .iter()
                .map(|x| self.trans.get(&x).unwrap()),
        )
        .map(|x| x.clone())
        .collect()
    }

    fn epsilon_move(&self, state: &States) -> HashSet<States> {
        let mut r: HashSet<States> = HashSet::new();

        self.epsilon_move_internal(state, &mut r);

        r.iter()
            .filter(|&x| x != state && (self.end.contains(x) || self.has_nontrivial_edge(x)))
            .map(|x| x.clone())
            .collect()
    }

    fn epsilon_move_internal(&self, state: &States, visited: &mut HashSet<States>) {
        if visited.contains(state) {
            return;
        }
        visited.insert(state.clone());

        for e in self
            .trans
            .get(state)
            .unwrap()
            .iter()
            .filter(|x| x.matches().is_none())
        {
            self.epsilon_move_internal(e.next_node(), visited);
        }
    }

    fn has_epsilon_edge(&self, state: &States) -> bool {
        self.trans
            .get(state)
            .unwrap()
            .iter()
            .any(|x| x.matches().is_none())
    }

    fn has_nontrivial_edge(&self, state: &States) -> bool {
        self.trans
            .get(state)
            .unwrap()
            .iter()
            .any(|x| x.matches().is_some())
    }
}

impl fmt::Display for TransTable {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        writeln!(f, "TransTable(start: {})", self.start.iter().join(","))?;

        // dump states
        let mut states = self.states.iter().map(|x| x.clone()).collect::<Vec<States>>();
        states.sort();

        for state in states.iter() {
            if self.end.contains(state) {
                writeln!(f, "\tState {}*", state.iter().join(","))?;
            } else {
                writeln!(f, "\tState {}", state.iter().join(","))?;
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
                        edge.next_node().iter().join(",")
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
            let mut states: Vec<States> = $table.epsilon_move($state).iter().map(|x| x.clone()).collect();
            let mut expect = $expect;

            states.sort();
            expect.sort();

            assert_eq!(states, expect);
        }};
    }

    #[test]
    fn test_cut_epsilon() {
        let r: RegexItem = r#"(a|b)+c"#.into();
        let mut t = TransTable::from_nfa(&r.nfa_graph());
        t.as_dfa();
        t.reset_state_mark();
        assert_eq!(t.state_count(), 4);
        assert_eq!(t.edge_count(), 8);

        let r: RegexItem = r#"([ab]+|c*)?"#.into();
        let mut t = TransTable::from_nfa(&r.nfa_graph());
        t.as_dfa();
        t.reset_state_mark();
        assert_eq!(t.state_count(), 4);
        assert_eq!(t.edge_count(), 8);

        let r: RegexItem = r#"(c|[a-z])+"#.into();
        let mut t = TransTable::from_nfa(&r.nfa_graph());
        t.as_dfa();
        t.reset_state_mark();
    }

    #[test]
    fn test_epsilon_move() {
        let r: RegexItem = r#"(a|b)+c"#.into();
        let mut t = TransTable::from_nfa(&r.nfa_graph());
        t.reset_state_mark();
        assert_eq!(t.states.len(), 8);
        assert_move!(t, &set![0], vec![set![2], set![4]]);
        assert_move!(t, &set![3], vec![set![6], set![2], set![4]]);
        assert_move!(t, &set![5], vec![set![6], set![2], set![4]]);
        assert_move!(t, &set![1], vec![set![6], set![2], set![4]]);
        assert_move!(t, &set![6], vec![]);
        assert_move!(t, &set![7], vec![]);

        let r: RegexItem = r#"[-c]*"#.into();
        let mut t = TransTable::from_nfa(&r.nfa_graph());
        t.reset_state_mark();
        assert_eq!(t.states.len(), 6);
        assert_move!(t, &set![0], vec![set![1], set![2], set![4]]);
        assert_move!(t, &set![1], vec![set![2], set![4]]);
        assert_move!(t, &set![2], vec![]);
        assert_move!(t, &set![4], vec![]);
        assert_move!(t, &set![3], vec![set![1], set![2], set![4]]);
        assert_move!(t, &set![5], vec![set![1], set![2], set![4]]);

        let r: RegexItem = r#"([ab]+|c*)?"#.into();
        let mut t = TransTable::from_nfa(&r.nfa_graph());
        t.reset_state_mark();
        assert_eq!(t.states.len(), 10);
        assert_move!(t, &set![0], vec![set![1], set![4], set![6], set![8]]);
        assert_move!(t, &set![2], vec![set![4], set![6]]);
        assert_move!(t, &set![8], vec![set![1]]);
        assert_move!(t, &set![5], vec![set![1], set![4], set![6]]);
    }
}
