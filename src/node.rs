use std::sync::atomic::{AtomicUsize, Ordering};

static ID_SEQ: AtomicUsize = AtomicUsize::new(0);

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
    NotCharacter(u8),
    NotRange(u8, u8),
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
            EdgeMatches::NotCharacter(c) => format!("Not {}", display(c)),
            EdgeMatches::NotRange(s, e) => format!("Not {}-{}", display(s), display(e)),
        }
    }
}

#[derive(Clone, Debug)]
pub struct Edge {
    matches: Option<EdgeMatches>,
    next_node: usize,
}

impl Edge {
    pub fn epsilon(next_node: usize) -> Edge {
        Edge {
            matches: None,
            next_node,
        }
    }

    pub fn new(dest: usize, matches: Option<EdgeMatches>) -> Edge {
        Edge {
            matches: matches,
            next_node: dest,
        }
    }

    pub fn matches(&self) -> &Option<EdgeMatches> {
        &self.matches
    }

    pub fn next_node(&self) -> usize {
        self.next_node
    }

    pub fn match_character(&self, c: u8) -> bool {
        match self.matches {
            Some(EdgeMatches::Character(ch)) => c == ch,
            Some(EdgeMatches::NotCharacter(ch)) => c != ch,
            Some(EdgeMatches::CharacterRange(s, e)) => c >= s && c <= e,
            Some(EdgeMatches::NotRange(s, e)) => c < s || c > e,
            None => false,
        }
    }

    pub fn intersect(&self, e: &Edge) -> bool {
        if self.matches.is_none() || e.matches.is_none() {
            return false;
        }

        let lhs = self.matches.as_ref().unwrap();
        let rhs = e.matches.as_ref().unwrap();

        match (lhs, rhs) {
            (EdgeMatches::Character(c), _) => e.match_character(*c),
            (EdgeMatches::NotCharacter(c), _) => !e.match_character(*c),
            (EdgeMatches::NotRange(_, _), EdgeMatches::NotRange(_, _)) => true,
            (EdgeMatches::CharacterRange(ls, le), EdgeMatches::CharacterRange(rs, re)) => range_intersect(ls, le, rs, re),
            (EdgeMatches::NotRange(ls, le), EdgeMatches::CharacterRange(rs, re)) => ls == rs && le == re,
            _ => e.intersect(self),
        }
    }
}

fn range_intersect(ls: &u8, le: &u8, rs: &u8, re: &u8) -> bool {
    (rs <= ls && ls <= re) ||
    (rs <= le && le <= re) ||
    (ls <= rs && rs <= le) ||
    (ls <= re && re <= le)
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

    pub fn connect(&mut self, dest: usize, matches: Option<EdgeMatches>) {
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
        let l = Edge::new(0, None);
        let r = Edge::new(0, Some(EdgeMatches::Character(b'c')));
        assert_eq!(l.intersect(&r), false);
        assert_eq!(r.intersect(&l), false);

        let l = Edge::new(0, Some(EdgeMatches::Character(b'c')));
        assert_eq!(l.intersect(&r), true);
        assert_eq!(r.intersect(&l), true);

        let l = Edge::new(0, Some(EdgeMatches::CharacterRange(b'a', b'z')));
        assert_eq!(l.intersect(&r), true);
        assert_eq!(r.intersect(&l), true);

        let r = Edge::new(0, Some(EdgeMatches::NotCharacter(b'c')));
        assert_eq!(l.intersect(&r), false);
        assert_eq!(r.intersect(&l), false);

        let r = Edge::new(0, Some(EdgeMatches::CharacterRange(b'0', b'9')));
        assert_eq!(l.intersect(&r), false);
        assert_eq!(r.intersect(&l), false);

        let r = Edge::new(0, Some(EdgeMatches::CharacterRange(b'd', b'f')));
        assert_eq!(l.intersect(&r), true);
        assert_eq!(r.intersect(&l), true);

        let r = Edge::new(0, Some(EdgeMatches::CharacterRange(b'A', b'f')));
        assert_eq!(l.intersect(&r), true);
        assert_eq!(r.intersect(&l), true);
    }

    #[test]
    fn test_edge_match_character() {
        let edge = Edge::new(0, None);
        assert_eq!(edge.match_character(b'c'), false);

        let edge = Edge::new(0, Some(EdgeMatches::Character(b'c')));
        assert_eq!(edge.match_character(b'c'), true);
        assert_eq!(edge.match_character(b'd'), false);

        let edge = Edge::new(0, Some(EdgeMatches::CharacterRange(b'3', b'5')));
        assert_eq!(edge.match_character(b'2'), false);
        assert_eq!(edge.match_character(b'3'), true);
        assert_eq!(edge.match_character(b'4'), true);
        assert_eq!(edge.match_character(b'5'), true);
        assert_eq!(edge.match_character(b'6'), false);

        let edge = Edge::new(0, Some(EdgeMatches::NotCharacter(b'3')));
        assert_eq!(edge.match_character(b'2'), true);
        assert_eq!(edge.match_character(b'3'), false);
        assert_eq!(edge.match_character(b'4'), true);

        let edge = Edge::new(0, Some(EdgeMatches::NotRange(b'3', b'5')));
        assert_eq!(edge.match_character(b'2'), true);
        assert_eq!(edge.match_character(b'3'), false);
        assert_eq!(edge.match_character(b'4'), false);
        assert_eq!(edge.match_character(b'5'), false);
        assert_eq!(edge.match_character(b'6'), true);
    }
}

