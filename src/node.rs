
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

    pub fn nodes(&mut self) -> (&mut Node, &mut Node) {
        (&mut self.start, &mut self.end)
    }

    pub fn start(&mut self) -> &mut Node {
        &mut self.start
    }

    pub fn start_id(&self) -> usize {
        self.start.id()
    }

    pub fn end_id(&self) -> usize {
        self.end.id()
    }
}

pub struct Edge {
    character: Option<u8>,
    next_node: usize,
}

impl Edge {
    pub fn epsilon(next_node: usize) -> Edge {
        Edge {
            character: None,
            next_node,
        }
    }

    pub fn new(dest: usize, ch: Option<u8>) -> Edge {
        Edge {
            character: ch,
            next_node: dest,
        }
    }
}

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

    pub fn id(&self) -> usize {
        self.id
    }

    pub fn connect(&mut self, dest: usize, ch: Option<u8>) {
        self.append_edge(Edge::new(dest, ch));
    }

    pub fn append_edge(&mut self, edge: Edge) {
        self.edges.push(edge);
    }

    pub fn edges(&self) -> &Vec<Edge> {
        &self.edges
    }
}


