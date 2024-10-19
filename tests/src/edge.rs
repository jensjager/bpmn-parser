pub struct Edge {
    from: usize,
    to: usize,
}

impl Edge {
    pub fn new(from: usize, to: usize) -> Edge {
        Edge { from, to }
    }

    pub fn from(&self) -> usize {
        self.from
    }

    pub fn to(&self) -> usize {
        self.to
    }
}
