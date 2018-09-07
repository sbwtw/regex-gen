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

#[derive(Clone)]
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

    pub fn matches(&self) -> Option<u8> {
        self.character
    }

    pub fn next_node(&self) -> usize {
        self.next_node
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
