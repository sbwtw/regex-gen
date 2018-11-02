use std::convert::From;
use std::iter::Peekable;
use std::str::Chars;
use std::string::ToString;

use node::*;

#[derive(Debug, PartialEq)]
pub enum RegexUnit {
    Character(u8),
    CharacterRange(u8, u8),
    NotCharacter(u8),
    NotUnits(Vec<RegexUnit>),
    UnitChoice(Vec<RegexUnit>),
    ItemList(Vec<RegexItem>),
    ItemChoice(Vec<RegexItem>),
}

#[derive(Debug, PartialEq)]
pub enum RegexAnnotation {
    StandAlone,
    OneOrZero,   // '?'
    GreaterZero, // '+'
    AnyOccurs,   // '*'
}

#[derive(Debug, PartialEq)]
pub struct RegexItem {
    unit: RegexUnit,
    annotation: RegexAnnotation,
}

impl<'s> From<&'s str> for RegexItem {
    fn from(s: &'s str) -> RegexItem {
        RegexParser {
            input: s.chars().peekable(),
        }.parse()
        .unwrap()
    }
}

impl ToString for RegexUnit {
    fn to_string(&self) -> String {
        let mut r = String::new();

        match self {
            RegexUnit::Character(c) => match c {
                b'\n' => r.push_str("\\n"),
                _ => r.push(*c as char),
            }
            RegexUnit::CharacterRange(s, e) => {
                r.push(*s as char);
                r.push('-');
                r.push(*e as char);
            }
            RegexUnit::NotCharacter(c) => {
                match c {
                    b'\n' => r.push('.'),
                    _ => {
                        r.push_str("[^");
                        r.push(*c as char);
                        r.push(']');
                    }
                }
            }
            RegexUnit::NotUnits(list) => {
                r.push_str("[^");
                for i in list {
                    r.push_str(&i.to_string());
                }
                r.push(']');
            }
            RegexUnit::UnitChoice(list) => {
                r.push('[');
                for i in list {
                    r.push_str(&i.to_string());
                }
                r.push(']');
            }
            RegexUnit::ItemChoice(list) => {
                let mut it = list.iter();

                r.push('(');
                if let Some(item) = it.next() {
                    r.push_str(&item.to_string());
                }
                for item in it {
                    r.push('|');
                    r.push_str(&item.to_string());
                }
                r.push(')');
            }
            RegexUnit::ItemList(list) => {
                for i in list {
                    r.push_str(&i.to_string());
                }
            }
        }

        r
    }
}

impl ToString for RegexItem {
    fn to_string(&self) -> String {
        let mut r = String::new();

        r.push_str(&self.unit.to_string());

        match self.annotation {
            RegexAnnotation::AnyOccurs => r.push('*'),
            RegexAnnotation::OneOrZero => r.push('?'),
            RegexAnnotation::GreaterZero => r.push('+'),
            _ => {}
        }

        r
    }
}

impl RegexUnit {
    fn nfa_graph(&self) -> NFAGraph {
        match self {
            &RegexUnit::Character(c) => {
                let mut graph = NFAGraph::new();
                {
                    let end_id = graph.end_id();
                    let (start, _) = graph.nodes_mut();

                    start.connect(set![end_id], Some(EdgeMatches::Character(c)));
                }

                graph
            }
            &RegexUnit::CharacterRange(s, e) => {
                let mut graph = NFAGraph::new();
                {
                    let end_id = graph.end_id();
                    let (start, _) = graph.nodes_mut();

                    start.connect(set![end_id], Some(EdgeMatches::CharacterRange(s, e)));
                }

                graph
            }
            &RegexUnit::NotCharacter(c) => {
                let mut graph = NFAGraph::new();
                {
                    let end_id = graph.end_id();
                    let (start, _) = graph.nodes_mut();

                    start.connect(set![end_id], Some(EdgeMatches::Not(vec![EdgeMatches::Character(c)])));
                }

                graph
            }
            &RegexUnit::NotUnits(ref list) => {
                let mut graph = NFAGraph::new();
                {
                    let end_id = graph.end_id();
                    let (start, _) = graph.nodes_mut();

                    let mut matches = vec![];
                    for item in list {
                        match item {
                            RegexUnit::Character(c) =>
                                matches.push(EdgeMatches::Character(*c)),
                            RegexUnit::CharacterRange(s, e) =>
                                matches.push(EdgeMatches::CharacterRange(*s, *e)),
                            _ => unimplemented!()
                        }
                    }

                    start.connect(set![end_id], Some(EdgeMatches::Not(matches)));
                }

                graph
            }
            &RegexUnit::UnitChoice(ref list) => {
                let mut sub_graphs = vec![];
                let mut graph = NFAGraph::new();
                let end_id = graph.end_id();
                {
                    let (start, _) = graph.nodes_mut();

                    for item in list {
                        let mut g = item.nfa_graph();

                        // connect start to sub graph start
                        start.connect(set![g.start_id()], None);
                        // connect sub graph to our end
                        g.end_mut().connect(set![end_id], None);

                        sub_graphs.push(g);
                    }
                }

                // merge sub_graphs to graph
                for g in sub_graphs {
                    graph.append_sub_graph(g);
                }

                graph
            }
            &RegexUnit::ItemList(ref list) => {
                assert!(list.len() > 0);
                let mut gs: Vec<NFAGraph> = list.iter().map(|x| x.nfa_graph()).collect();
                let mut graph =
                    NFAGraph::from_id(gs[0].start_id(), gs.last_mut().unwrap().end_id());

                for i in 0..(gs.len() - 1) {
                    let id = gs[i + 1].start_id();
                    gs[i].end_mut().connect(set![id], None);
                }

                // merge
                for g in gs {
                    graph.append_sub_graph(g);
                }

                graph
            }
            &RegexUnit::ItemChoice(ref list) => {
                let mut sub_graphs = vec![];
                let mut graph = NFAGraph::new();
                let end_id = graph.end_id();
                {
                    let (start, _) = graph.nodes_mut();

                    for item in list {
                        let mut g = item.nfa_graph();

                        // connect start to sub graph start
                        start.connect(set![g.start_id()], None);
                        // connect sub graph to our end
                        g.end_mut().connect(set![end_id], None);

                        sub_graphs.push(g);
                    }
                }

                // merge sub_graphs to graph
                for g in sub_graphs {
                    graph.append_sub_graph(g);
                }

                graph
            }
        }
    }
}

impl RegexItem {
    pub fn nfa_graph(&self) -> NFAGraph {
        let mut graph = self.unit.nfa_graph();
        let end_id = graph.end_id();
        let start_id = graph.start_id();

        match self.annotation {
            RegexAnnotation::OneOrZero => {
                // `?`
                graph.start_mut().connect(set![end_id], None);
            }
            RegexAnnotation::GreaterZero => {
                // `+`
                graph.end_mut().connect(set![start_id], None);
            }
            RegexAnnotation::AnyOccurs => {
                // '*'
                graph.start_mut().connect(set![end_id], None);
                graph.end_mut().connect(set![start_id], None);
            }
            RegexAnnotation::StandAlone => {}
        }

        graph
    }
}

type RegexParserError = ();
type RegexParserResult = Result<RegexItem, RegexParserError>;

struct RegexParser<'s> {
    input: Peekable<Chars<'s>>,
}

impl<'s> RegexParser<'s> {
    fn parse(&mut self) -> RegexParserResult {
        let mut items = vec![];

        while let Ok(item) = self.dispatch() {
            items.push(item);
        }

        assert_eq!(self.parse_annotation(), RegexAnnotation::StandAlone);

        Ok(RegexItem {
            unit: RegexUnit::ItemList(items),
            annotation: RegexAnnotation::StandAlone,
        })
    }

    fn dispatch(&mut self) -> RegexParserResult {
        if let Some(c) = self.input.peek().map(|x| x.clone()) {
            match c {
                '[' => self.parse_character_group(),
                '(' => self.parse_item_group(),
                _ => self.parse_character(),
            }
        } else {
            Err(())
        }
    }

    fn parse_character(&mut self) -> RegexParserResult {

        match self.input.peek().map(|x| x.clone()) {
            Some('\\') => self.parse_character_escape(),
            Some('.') => {
                self.input.next();

                Ok(RegexItem {
                    unit: RegexUnit::NotCharacter(b'\n'),
                    annotation: self.parse_annotation(),
                })
            }
            Some(c) => {
                self.input.next();

                Ok(RegexItem {
                    unit: RegexUnit::Character(c as u8),
                    annotation: self.parse_annotation(),
                })
            }
            _ => return Err(())
        }
    }

    fn parse_character_escape(&mut self) -> RegexParserResult {
        assert_eq!(Some('\\'), self.input.next());

        match self.input.next() {
            Some('d') => {
                Ok(RegexItem {
                    unit: RegexUnit::CharacterRange(b'0', b'9'),
                    annotation: self.parse_annotation(),
                })
            }
            Some(c) => {
                Ok(RegexItem {
                    unit: RegexUnit::Character(c as u8),
                    annotation: self.parse_annotation(),
                })
            }
            _ => return Err(()),
        }
    }

    fn parse_character_group(&mut self) -> RegexParserResult {
        assert_eq!(Some('['), self.input.next());
        let mut items = vec![];
        let mut not = false;

        // special process for '^'
        if let Some('^') = self.input.peek() {
            self.input.next();

            not = true;
        }

        // special process for '-'
        if let Some('-') = self.input.peek() {
            self.input.next();

            items.push(RegexUnit::Character(b'-'));
        }

        loop {
            match self.input.next().map(|x| x.clone()) {
                Some('\\') => match self.input.next() {
                    Some('d') => {
                        items.push(RegexUnit::CharacterRange(b'0', b'9'));
                    }
                    Some(c) => {
                        items.push(RegexUnit::Character(c as u8));
                    }
                    _ => return Err(()),
                },
                Some('a') => {
                    if let Some('-') = self.input.peek() {
                        self.input.next();
                        match self.input.next() {
                            Some('z') => items.push(RegexUnit::CharacterRange(b'a', b'z')),
                            _ => return Err(()),
                        }
                    } else {
                        items.push(RegexUnit::Character(b'a'));
                    }
                }
                Some('A') => {
                    if let Some('-') = self.input.peek() {
                        self.input.next();
                        match self.input.next() {
                            Some('Z') => items.push(RegexUnit::CharacterRange(b'A', b'Z')),
                            _ => return Err(()),
                        }
                    } else {
                        items.push(RegexUnit::Character(b'A'));
                    }
                }
                Some('0') => {
                    if let Some('-') = self.input.peek() {
                        self.input.next();
                        match self.input.next() {
                            Some('9') => items.push(RegexUnit::CharacterRange(b'0', b'9')),
                            _ => return Err(()),
                        }
                    } else {
                        items.push(RegexUnit::Character(b'0'));
                    }
                }
                Some(']') => {
                    let unit = if not {
                        RegexUnit::NotUnits(items)
                    } else {
                        RegexUnit::UnitChoice(items)
                    };

                    return Ok(RegexItem {
                        unit,
                        annotation: self.parse_annotation(),
                    })
                }
                Some(c) => {
                    items.push(RegexUnit::Character(c as u8));
                }
                None => return Err(()),
            }
        }
    }

    fn parse_item_group(&mut self) -> RegexParserResult {
        assert_eq!(Some('('), self.input.next());
        let mut items = vec![];
        let mut buffer = String::new();

        loop {
            match self.input.next() {
                Some(')') => {
                    items.push((&buffer[..]).into());

                    return Ok(RegexItem {
                        unit: RegexUnit::ItemChoice(items),
                        annotation: self.parse_annotation(),
                    });
                }
                Some('|') => {
                    items.push((&buffer[..]).into());
                    buffer.clear();
                }
                Some(c) => buffer.push(c),
                None => return Err(()),
            }
        }
    }

    fn parse_annotation(&mut self) -> RegexAnnotation {
        let r = match self.input.peek() {
            Some('?') => RegexAnnotation::OneOrZero,
            Some('+') => RegexAnnotation::GreaterZero,
            Some('*') => RegexAnnotation::AnyOccurs,
            _ => return RegexAnnotation::StandAlone,
        };

        self.input.next();
        r
    }
}

#[cfg(test)]
mod test {

    use regex_gen::*;
    use transtable::*;

    #[test]
    fn test_print_graph() {
        let r: RegexItem = r#"abc"#.into();
        let t = TransTable::from_nfa(&r.nfa_graph());
        assert_eq!(t.state_count(), 6);
        assert_eq!(t.edge_count(), 5);

        let r: RegexItem = r#"[bc]"#.into();
        let t = TransTable::from_nfa(&r.nfa_graph());
        assert_eq!(t.edge_count(), 6);

        let r: RegexItem = r#"[bc]+"#.into();
        let t = TransTable::from_nfa(&r.nfa_graph());
        assert_eq!(t.edge_count(), 7);

        let r: RegexItem = r#"(a*|[bc]?d)+"#.into();
        let t = TransTable::from_nfa(&r.nfa_graph());
        assert_eq!(t.state_count(), 12);
        assert_eq!(t.edge_count(), 17);

        let r: RegexItem = r#"\d+"#.into();
        let t = TransTable::from_nfa(&r.nfa_graph());
        assert_eq!(t.state_count(), 2);
        assert_eq!(t.edge_count(), 2);

        let r: RegexItem = r#"(.+|\d+)?"#.into();
        let t = TransTable::from_nfa(&r.nfa_graph());
        assert_eq!(t.state_count(), 6);
        assert_eq!(t.edge_count(), 9);

        let r: RegexItem = r#"[^a-z5]"#.into();
        let t = TransTable::from_nfa(&r.nfa_graph());
        assert_eq!(t.state_count(), 2);
        assert_eq!(t.edge_count(), 1);

        let r: RegexItem = r#"[^a-z5]+"#.into();
        let t = TransTable::from_nfa(&r.nfa_graph());
        assert_eq!(t.state_count(), 2);
        assert_eq!(t.edge_count(), 2);
    }

    #[test]
    fn test_parse() {
        let r1: RegexItem = r#"a[-a\\bd\[\]\d]+"#.into();
        let r2: RegexItem = r#"a[-a\\bd\[\]0-9]+"#.into();
        assert_eq!(r1, r2);

        let s = r#"a(bc|de)f"#;
        let r: RegexItem = s.into();
        assert_eq!(r.to_string(), "a(bc|de)f".to_string());
        assert_eq!(r.to_string(), s);

        let s = r#"a(b+[cde]*|de)f"#;
        let r: RegexItem = s.into();
        assert_eq!(r.to_string(), "a(b+[cde]*|de)f".to_string());
        assert_eq!(r.to_string(), s);

        let s = r#".+"#;
        let r: RegexItem = s.into();
        assert_eq!(r.to_string(), s);
    }
}
