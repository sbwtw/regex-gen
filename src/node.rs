use std::sync::atomic::{AtomicUsize, Ordering};
use std::collections::BTreeSet;

static ID_SEQ: AtomicUsize = AtomicUsize::new(0);

pub type States = BTreeSet<usize>;

pub struct NFAGraph {
    start: Node,
    end: Node,

    sub_graphs: Vec<NFAGraph>,
}

impl NFAGraph {
    pub fn new() -> NFAGraph {
        NFAGraph {
            start: Node::new(),
            end: Node::new(),

            sub_graphs: vec![],
        }
    }

    pub fn from_node(start: Node, end: Node) -> NFAGraph {
        NFAGraph {
            start,
            end,

            sub_graphs: vec![],
        }
    }

    pub fn from_id(start: usize, end: usize) -> NFAGraph {
        NFAGraph {
            start: Node::from_id(start),
            end: Node::from_id(end),

            sub_graphs: vec![],
        }
    }

    #[cfg(test)]
    pub fn edge_count(&self) -> usize {
        self.sub_graphs
            .iter()
            .map(|x| x.edge_count())
            .sum::<usize>()
            + self.start.edge_count()
            + self.end.edge_count()
    }

    pub fn nodes(&self) -> (&Node, &Node) {
        (&self.start, &self.end)
    }

    pub fn nodes_mut(&mut self) -> (&mut Node, &mut Node) {
        (&mut self.start, &mut self.end)
    }

    pub fn sub_graphs(&self) -> &Vec<NFAGraph> {
        &self.sub_graphs
    }

    pub fn start_mut(&mut self) -> &mut Node {
        &mut self.start
    }

    pub fn end_mut(&mut self) -> &mut Node {
        &mut self.end
    }

    pub fn start_id(&self) -> usize {
        self.start.id()
    }

    pub fn end_id(&self) -> usize {
        self.end.id()
    }

    pub fn append_sub_graph(&mut self, g: NFAGraph) {
        self.sub_graphs.push(g);
    }
}

#[derive(Clone, Debug)]
pub enum EdgeMatches {
    Character(u8),
    CharacterRange(u8, u8),
    Not(Vec<EdgeMatches>),
}

impl EdgeMatches {
    fn match_character(&self, c: u8) -> bool {
        match self {
            &EdgeMatches::Character(ch) => c == ch,
            &EdgeMatches::CharacterRange(s, e) => c >= s && c <= e,
            &EdgeMatches::Not(ref list) => !list.iter().any(|x| x.match_character(c)),
        }
    }

    fn intersect(&self, rhs: &EdgeMatches) -> bool {
        match (self, rhs) {
            // 定义在语言上的字符集是无限的，那么不可能有两个 Not 集合是不相交的。
            // 在边处理的时候，需要把两个 Not 集合拆分并分别表示。
            (EdgeMatches::Not(_), EdgeMatches::Not(_)) => true,
            (EdgeMatches::Character(c), _) => rhs.match_character(*c),
            (EdgeMatches::CharacterRange(ls, le), EdgeMatches::CharacterRange(rs, re)) => range_intersect(ls, le, rs, re),
            (EdgeMatches::Not(_), EdgeMatches::CharacterRange(s, e)) => (*s..*e).all(|x| self.match_character(x)),
            // swap rhs & lhs
            _ => rhs.intersect(self),
        }
    }
}

fn range_intersect(ls: &u8, le: &u8, rs: &u8, re: &u8) -> bool {
    (rs <= ls && ls <= re) ||
    (rs <= le && le <= re) ||
    (ls <= rs && rs <= le) ||
    (ls <= re && re <= le)
}

#[inline]
fn display(c: &u8) -> String {
    match *c {
        b'\n' => "<br>".to_string(),
        _ => format!("'{}'", *c as char),
    }
}

impl ToString for EdgeMatches {
    fn to_string(&self) -> String {
        match self {
            EdgeMatches::Character(c) => format!("{}", display(c)),
            EdgeMatches::CharacterRange(s, e) => format!("{}-{}", display(s), display(e)),
            EdgeMatches::Not(list) => {
                let mut s = "Not ".to_string();
                let mut iter = list.iter();

                if let Some(item) = iter.next() {
                    s.push_str(&item.to_string());
                }

                for item in iter.next() {
                    s.push_str(", ");
                    s.push_str(&item.to_string());
                }

                s
            }
        }
    }
}

#[derive(Clone, Debug)]
pub struct Edge {
    matches: Option<EdgeMatches>,
    next_node: States,
}

impl Edge {
    pub fn epsilon(next_node: States) -> Edge {
        Edge {
            matches: None,
            next_node,
        }
    }

    pub fn new(dest: States, matches: Option<EdgeMatches>) -> Edge {
        Edge {
            matches: matches,
            next_node: dest,
        }
    }

    pub fn matches(&self) -> &Option<EdgeMatches> {
        &self.matches
    }

    pub fn next_node(&self) -> &States {
        &self.next_node
    }

    pub fn match_character(&self, c: u8) -> bool {
        self.matches.as_ref().map_or(false, |ref x| x.match_character(c))
    }

    pub fn intersect(&self, e: &Edge) -> bool {
        match (self.matches.as_ref(), e.matches.as_ref()) {
            (Some(lhs), Some(rhs)) => lhs.intersect(&rhs),
            _ => false,
        }
    }
}

#[derive(Clone)]
pub struct Node {
    id: usize,
    edges: Vec<Edge>,
}

impl Node {
    pub fn new() -> Node {
        Node {
            id: ID_SEQ.fetch_add(1, Ordering::SeqCst),
            edges: vec![],
        }
    }

    pub fn from_id(id: usize) -> Node {
        Node { id, edges: vec![] }
    }

    #[cfg(test)]
    pub fn edge_count(&self) -> usize {
        self.edges.len()
    }

    pub fn id(&self) -> usize {
        self.id
    }

    pub fn connect(&mut self, dest: States, matches: Option<EdgeMatches>) {
        self.append_edge(Edge::new(dest, matches));
    }

    pub fn append_edge(&mut self, edge: Edge) {
        self.edges.push(edge);
    }

    pub fn edges(&self) -> &Vec<Edge> {
        &self.edges
    }
}

#[cfg(test)]
mod test {
    use node::*;

    #[test]
    fn test_edge_intersect() {
        let l = Edge::new(set![0], None);
        let r = Edge::new(set![0], Some(EdgeMatches::Character(b'c')));
        assert_eq!(l.intersect(&r), false);
        assert_eq!(r.intersect(&l), false);

        let l = Edge::new(set![0], Some(EdgeMatches::Character(b'c')));
        assert_eq!(l.intersect(&r), true);
        assert_eq!(r.intersect(&l), true);

        let l = Edge::new(set![0], Some(EdgeMatches::CharacterRange(b'a', b'z')));
        assert_eq!(l.intersect(&r), true);
        assert_eq!(r.intersect(&l), true);

        let r = Edge::new(set![0], Some(EdgeMatches::Not(vec![EdgeMatches::Character(b'c')])));
        assert_eq!(l.intersect(&r), false);
        assert_eq!(r.intersect(&l), false);

        let r = Edge::new(set![0], Some(EdgeMatches::CharacterRange(b'0', b'9')));
        assert_eq!(l.intersect(&r), false);
        assert_eq!(r.intersect(&l), false);

        let r = Edge::new(set![0], Some(EdgeMatches::CharacterRange(b'd', b'f')));
        assert_eq!(l.intersect(&r), true);
        assert_eq!(r.intersect(&l), true);

        let r = Edge::new(set![0], Some(EdgeMatches::CharacterRange(b'A', b'f')));
        assert_eq!(l.intersect(&r), true);
        assert_eq!(r.intersect(&l), true);
    }

    #[test]
    fn test_edge_match_character() {
        let edge = Edge::new(set![0], None);
        assert_eq!(edge.match_character(b'c'), false);

        let edge = Edge::new(set![0], Some(EdgeMatches::Character(b'c')));
        assert_eq!(edge.match_character(b'c'), true);
        assert_eq!(edge.match_character(b'd'), false);

        let edge = Edge::new(set![0], Some(EdgeMatches::CharacterRange(b'3', b'5')));
        assert_eq!(edge.match_character(b'2'), false);
        assert_eq!(edge.match_character(b'3'), true);
        assert_eq!(edge.match_character(b'4'), true);
        assert_eq!(edge.match_character(b'5'), true);
        assert_eq!(edge.match_character(b'6'), false);

        let edge = Edge::new(set![0], Some(EdgeMatches::Not(vec![EdgeMatches::Character(b'3')])));
        assert_eq!(edge.match_character(b'2'), true);
        assert_eq!(edge.match_character(b'3'), false);
        assert_eq!(edge.match_character(b'4'), true);

        let edge = Edge::new(set![0], Some(EdgeMatches::Not(vec![EdgeMatches::CharacterRange(b'3', b'5')])));
        assert_eq!(edge.match_character(b'2'), true);
        assert_eq!(edge.match_character(b'3'), false);
        assert_eq!(edge.match_character(b'4'), false);
        assert_eq!(edge.match_character(b'5'), false);
        assert_eq!(edge.match_character(b'6'), true);
    }
}

