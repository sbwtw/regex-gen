
use transtable::TransTable;

pub trait ToDotGraph {
    fn to_dot_graph(&self) -> String;
}

impl ToDotGraph for TransTable {
    fn to_dot_graph(&self) -> String {
        let mut s = String::new();

        s.push_str("digraph {\n");
        s.push_str("\trankdir=LR;\n");
        s.push_str(&format!("\tstart -> {};\n", self.start_id()));

        for (state, edges) in self.trans_map().iter() {
            for edge in edges.iter() {
                s.push_str(&format!("\t{} -> {} [label=\"{}\"];\n", state, edge.next_node(), edge.matches().as_ref().unwrap().to_string()));
            }
        }

        s.push_str("\tstart [shape=none,label=\"\",height=0,width=0]\n");

        for state in self.end_set().iter() {
            s.push_str(&format!("\t{} [peripheries=2]\n", state));
        }

        s.push_str("}\n");

        s
    }
}

#[cfg(test)]
mod test {

    use regex_gen::*;
    use transtable::*;
    //use dot_graph::*;

    #[test]
    fn test_dot_graph() {
        let r: RegexItem = r#"a([b\d]?c|d)+"#.into();
        let mut t = TransTable::from_nfa(&r.nfa_graph());
        t.as_dfa();

        //println!("{}", t.to_dot_graph());
    }
}

