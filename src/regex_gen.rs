
use std::io;
use std::io::Write;
use std::convert::From;
use std::str::Chars;
use std::iter::Peekable;
use std::string::ToString;

use super::CodeGenerator;
use node::*;

#[derive(Debug, PartialEq)]
pub enum RegexUnit {
    Character(u8),
    CharacterRange(u8, u8),
    UnitChoice(Vec<RegexUnit>),
    ItemList(Vec<RegexItem>),
    ItemChoice(Vec<RegexItem>),
}

#[derive(Debug, PartialEq)]
pub enum RegexAnnotation {
    StandAlone,
    OneOrZero,      // '?'
    GreaterZero,    // '+'
    AnyOccurs,      // '*'
}

#[derive(Debug, PartialEq)]
pub struct RegexItem {
    unit: RegexUnit,
    annotation: RegexAnnotation,
}

impl CodeGenerator for RegexItem {
    fn generate<W: Write>(&self, w: &mut W) -> io::Result<()> {
        // function begin
        writeln!(w, "fn match_regex<T: AsRef<str>>(s: T) -> bool {{")?;

        writeln!(w, "let input = s.chars().peekable();")?;

        // generate code

        // function end
        writeln!(w, "}}")?;

        Ok(())
    }
}

impl<'s> From<&'s str> for RegexItem {
    fn from(s: &'s str) -> RegexItem {
        RegexParser {
            input: s.chars().peekable(),
        }.parse().unwrap()
    }
}

impl RegexItem {
    pub fn first_characters_set(&self) -> Vec<u8> {
        self.unit.first_characters_set()
    }
}

impl RegexUnit {
    fn first_characters_set(&self) -> Vec<u8> {
        match self {
            &RegexUnit::Character(c) => vec![c],
            &RegexUnit::CharacterRange(s, e) => (s..(e + 1)).collect(),
            &RegexUnit::UnitChoice(ref list) =>
                list.iter()
                    .map(|x| x.first_characters_set())
                    .flatten()
                    .collect(),
            &RegexUnit::ItemChoice(ref list) =>
                list.iter()
                    .map(|x| x.first_characters_set())
                    .flatten()
                    .collect(),
            &RegexUnit::ItemList(ref list) => {
                let mut r = vec![];

                for item in list {
                    r.append(&mut item.first_characters_set());

                    if !matches!(item.annotation, RegexAnnotation::OneOrZero | RegexAnnotation::AnyOccurs) {
                        return r;
                    }
                }

                r
            }
        }
    }
}

impl ToString for RegexUnit {
    fn to_string(&self) -> String {
        let mut r = String::new();

        match self {
            RegexUnit::Character(c) => r.push(*c as char),
            RegexUnit::CharacterRange(s, e) => {
                r.push(*s as char);
                r.push('-');
                r.push(*e as char);
            },
            RegexUnit::UnitChoice(list) => {
                r.push('[');
                for i in list {
                    r.push_str(&i.to_string());
                }
                r.push(']');
            },
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
            },
            RegexUnit::ItemList(list) => {
                for i in list {
                    r.push_str(&i.to_string());
                }
            },

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
            _ => {},
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
                    let (start, _) = graph.nodes();

                    start.connect(end_id, Some(c));
                }

                graph
            },
            &RegexUnit::CharacterRange(s, e) => {
                let mut graph = NFAGraph::new();
                {
                    let end_id = graph.end_id();
                    let (start, _) = graph.nodes();

                    for c in s..(e + 1) {
                        start.connect(end_id, Some(c));
                    }
                }

                graph
            },
            &RegexUnit::UnitChoice(ref list) => {
                let mut sub_graphs = vec![];
                let mut graph = NFAGraph::new();
                let end_id = graph.end_id();
                {
                    let (start, _) = graph.nodes();

                    for item in list {
                        let mut g = item.nfa_graph();

                        // connect start to sub graph start
                        start.connect(g.start_id(), None);
                        // connect sub graph to our end
                        g.end_mut().connect(end_id, None);

                        sub_graphs.push(g);
                    }
                }

                // merge sub_graphs to graph
                for g in sub_graphs {
                    graph.append_sub_graph(g);
                }

                graph
            },
            &RegexUnit::ItemList(ref list) => {
                assert!(list.len() > 0);
                let mut gs: Vec<NFAGraph> = list.iter().map(|x| x.nfa_graph()).collect();
                let mut graph = NFAGraph::from_id(gs[0].start_id(), gs.last_mut().unwrap().end_id());

                for i in 0..(gs.len() - 1) {
                    let id = gs[i + 1].start_id();
                    gs[i].end_mut().connect(id, None);
                }

                // merge
                for g in gs {
                    graph.append_sub_graph(g);
                }

                graph
            },
            &RegexUnit::ItemChoice(ref list) => {
                let mut sub_graphs = vec![];
                let mut graph = NFAGraph::new();
                let end_id = graph.end_id();
                {
                    let (start, _) = graph.nodes();

                    for item in list {
                        let mut g = item.nfa_graph();

                        // connect start to sub graph start
                        start.connect(g.start_id(), None);
                        // connect sub graph to our end
                        g.end_mut().connect(end_id, None);

                        sub_graphs.push(g);
                    }
                }

                // merge sub_graphs to graph
                for g in sub_graphs {
                    graph.append_sub_graph(g);
                }

                graph
            },
        }
    }
}

impl RegexItem {
    pub fn nfa_graph(&self) -> NFAGraph {
        let mut graph = self.unit.nfa_graph();
        let end_id = graph.end_id();

        match self.annotation {
            RegexAnnotation::OneOrZero => {
                // `?`
                let end_id = graph.end_id().clone();
                graph.start_mut().connect(end_id, None);
            },
            RegexAnnotation::GreaterZero => {
                // `+`
                let edges: Vec<Option<u8>> =
                    graph.start_mut()
                        .edges()
                        .iter()
                        .filter(|x| x.next_node() == end_id && x.matches().is_some())
                        .map(|x| x.matches())
                        .collect();

                for edge in edges.iter() {
                    graph.end_mut().connect(end_id, *edge);
                }
            },
            RegexAnnotation::AnyOccurs => {
                // '*'
                let end_id = graph.end_id().clone();
                graph.start_mut().connect(end_id, None);

                let edges: Vec<Option<u8>> =
                    graph.start_mut()
                        .edges()
                        .iter()
                        .filter(|x| x.next_node() == end_id && x.matches().is_some())
                        .map(|x| x.matches())
                        .collect();

                for edge in edges.iter() {
                    graph.end_mut().connect(end_id, *edge);
                }
            },
            RegexAnnotation::StandAlone => {},
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
        if let Some(c) = self.input.peek().map(|x| x.clone())
        {
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
        let c = self.input.next().unwrap();

        Ok(RegexItem {
            unit: RegexUnit::Character(c as u8),
            annotation: self.parse_annotation(),
        })
    }

    fn parse_character_group(&mut self) -> RegexParserResult {
        assert_eq!(Some('['), self.input.next());
        let mut items = vec![];

        // special process for first '-'
        if let Some('-') = self.input.peek() {
            self.input.next();

            items.push(RegexUnit::Character(b'-'));
        }

        loop {
            match self.input.next().map(|x| x.clone()) {
                Some('\\') => {
                    match self.input.peek() {
                        Some('d') => {
                            self.input.next();

                            items.push(RegexUnit::CharacterRange(b'0', b'9'));
                        },
                        Some('\\') | Some('[') | Some(']') => {
                            items.push(RegexUnit::Character(self.input.next().unwrap() as u8));
                        },
                        _ => return Err(()),
                    }
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
                },
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
                },
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
                },
                Some(']') => {
                    return Ok(RegexItem {
                        unit: RegexUnit::UnitChoice(items),
                        annotation: self.parse_annotation(),
                    })
                },
                Some(c) => {
                    items.push(RegexUnit::Character(c as u8));
                },
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
                    })
                },
                Some('|') => {
                    items.push((&buffer[..]).into());
                    buffer.clear();
                },
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

    #[test]
    fn test_print_graph() {
        let r: RegexItem = r#"d[ef]g"#.into();
        let g = r.nfa_graph();

        println!("{:#?}", r);
        println!("{}", r.to_string());
        println!("{}", g);
    }

    #[test]
    fn test_parse() {
        let r1: RegexItem = r#"a[-a\\bd\[\]\d]+"#.into();
        let r2: RegexItem = r#"a[-a\\bd\[\]0-9]+"#.into();
        assert_eq!(r1, r2);

        let s = r#"a(bc|de)f"#;
        let r: RegexItem = s.into();
        assert_eq!(r.to_string(), s);

        let s = r#"a(b+[cde]*|de)f"#;
        let r: RegexItem = s.into();
        assert_eq!(r.to_string(), s);
    }

    #[test]
    fn test_write() {
        let r: RegexItem = r#"abc"#.into();

        r.generate(&mut io::stdout()).unwrap();
    }

    #[test]
    fn test_first_set() {
        let r: RegexItem = r#"a[bcd]ef"#.into();
        assert_eq!(r.first_characters_set(), vec![b'a']);

        let r: RegexItem = r#"[bcd]ef"#.into();
        assert_eq!(r.first_characters_set(), vec![b'b', b'c', b'd']);

        let r: RegexItem = r#"[bcd]*e?f"#.into();
        assert_eq!(r.first_characters_set(), vec![b'b', b'c', b'd', b'e', b'f']);

        let r: RegexItem = r#"(bc|de)ef"#.into();
        assert_eq!(r.first_characters_set(), vec![b'b', b'd']);

        let r: RegexItem = r#"(b?c|[de])ef"#.into();
        assert_eq!(r.first_characters_set(), vec![b'b', b'c', b'd', b'e']);

        let r: RegexItem = r#"[\d]"#.into();
        assert_eq!(r.first_characters_set(), vec![b'0', b'1', b'2', b'3', b'4', b'5', b'6', b'7', b'8', b'9']);

        let r: RegexItem = r#"[a-z]"#.into();
        let s = r.first_characters_set();
        assert!(s.contains(&b'a'));
        assert!(s.contains(&b'b'));
        assert!(s.contains(&b'h'));
        assert!(s.contains(&b'y'));
        assert!(s.contains(&b'z'));
        assert!(!s.contains(&b'0'));
        assert!(!s.contains(&b'A'));
    }
}

