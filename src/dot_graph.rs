
pub trait ToDotGraph {
    fn to_dot_graph(&self) -> String;
}

#[cfg(test)]
mod test {

    use regex_gen::*;
    use transtable::*;
    use dot_graph::*;

    #[test]
    fn test() {
        let r: RegexItem = r#"a([b\d]?c|d)+"#.into();
        let mut t = TransTable::from_nfa(&r.nfa_graph());
        t.cut_epsilon();

        println!("{}", t.to_dot_graph());
    }
}

