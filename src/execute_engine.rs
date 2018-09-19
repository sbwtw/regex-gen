
use transtable::TransTable;

pub struct ExecuteEngine {
    transtable: TransTable,
}

impl ExecuteEngine {
    pub fn with_transtable(transtable: TransTable) -> ExecuteEngine {
        ExecuteEngine {
            transtable,
        }
    }

    pub fn exact_match<T: AsRef<str>>(&self, s: T) -> bool {
        let mut s = s.as_ref().chars();
        let mut state = self.transtable.start_id();

        while let Some(c) = s.next() {
            let ref trans = self.transtable.trans_map().get(&state).unwrap();

            if let Some(e) = trans.iter().find(|x| x.match_character(c as u8)) {
                state = e.next_node();
            } else {
                return false;
            }
        }

        s.next().is_none() && self.transtable.end_set().contains(&state)
    }
}

#[cfg(test)]
mod test {
    use transtable::*;
    use execute_engine::*;
    use regex_gen::*;

    #[test]
    fn test_execute_engine() {
        let r: RegexItem = r#"a\d+b"#.into();
        let mut t = TransTable::from_nfa(&r.nfa_graph());
        t.cut_epsilon();

        let ee = ExecuteEngine::with_transtable(t);
        assert_eq!(ee.exact_match("a"), false);
        assert_eq!(ee.exact_match("ab"), false);
        assert_eq!(ee.exact_match("aab"), false);
        assert_eq!(ee.exact_match("a0"), false);
        assert_eq!(ee.exact_match("a0b"), true);
        assert_eq!(ee.exact_match("a0123456789b"), true);
    }
}

