struct Edge {
    from: usize,
    to: usize
}

pub fn new(from: usize, to: usize) -> Edge {
    Edge { from, to }
}
